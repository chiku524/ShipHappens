pub mod highlight;
pub mod motion;
pub mod ping;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};
use serde::{Deserialize, Serialize};

use crate::{
    core::{CRANE_JOB_ID, INTERACT_RADIUS, POWER_HOUR_JOB_ID},
    jobs::{apply_job_action, BreakerResult, JobActionResult, JobBoard, JobSystem, SmokeJobFlags},
    juice::{play_juice, CameraShake, FeedbackFlash, JuiceEvent},
    player::{CarryingFreight, Leaseholder, LocalPlayer, NetworkPlayer},
    rooms::{LayoutMarkerId, RoomRuntime},
    scoring::{PlayerScoreId, ScoreAction, ScoringService},
    tournament::{
        types::RoomId, types::SlotId, TournamentConfig, TournamentDirector, TournamentPhase,
        TournamentSnapshot,
    },
    Cli,
};

pub use motion::{attach_interact_motion, trigger_interact_motion, InteractMotion};
pub use ping::{breaker_panel_label, PingFeed, PingPlugin};

/// Bundles juice outputs so interact systems stay within Bevy's param limit.
#[derive(SystemParam)]
struct JuiceOut<'w, 's> {
    shake: ResMut<'w, CameraShake>,
    flash: ResMut<'w, FeedbackFlash>,
    audio: ResMut<'w, crate::audio_fx::AudioFxQueue>,
    vo: ResMut<'w, crate::audio_fx::VoQueue>,
    sparks: ResMut<'w, crate::juice::SparkQueue>,
    knockbacks: Query<'w, 's, &'static mut crate::player::Knockback>,
}

impl JuiceOut<'_, '_> {
    fn play(&mut self, event: JuiceEvent) {
        play_juice(event, &mut self.shake, &mut self.flash, &mut self.audio);
        if matches!(event, JuiceEvent::SortBad) {
            self.vo.push(crate::audio_fx::VoKind::SortWrong);
        }
    }

    fn play_at_station(
        &mut self,
        event: JuiceEvent,
        player: Entity,
        station_pos: Vec3,
        player_pos: Vec3,
    ) {
        self.play(event);
        if let Some(color) = crate::juice::juice_spark_color(event) {
            self.sparks.push(station_pos + Vec3::Y * 1.2, color);
        }
        if crate::juice::juice_applies_knockback(event) {
            if let Ok(mut knock) = self.knockbacks.get_mut(player) {
                *knock = crate::player::Knockback::shove_away_from(station_pos, player_pos, 9.0);
            }
        }
    }
}

#[derive(SystemParam)]
struct NetPresence<'w> {
    server: Option<Res<'w, RenetServer>>,
    client: Option<Res<'w, RenetClient>>,
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PingPlugin)
            .init_resource::<InteractPrompt>()
            .add_client_event::<InteractRequest>(Channel::Ordered)
            .add_observer(handle_interact_request)
            .add_systems(Startup, highlight::spawn_interact_highlight)
            .add_systems(
                Update,
                (
                    handle_drop_freight.run_if(in_state(crate::flow::AppScreen::Playing)),
                    (
                        handle_world_freight_pickup,
                        handle_local_interact,
                    )
                        .chain()
                        .run_if(in_state(crate::flow::AppScreen::Playing)),
                    show_interact_prompt.run_if(in_state(crate::flow::AppScreen::Playing)),
                    highlight::update_interact_highlight
                        .after(show_interact_prompt)
                        .run_if(in_state(crate::flow::AppScreen::Playing)),
                    motion::tick_interact_motion
                        .run_if(in_state(crate::flow::AppScreen::Playing)),
                ),
            );
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Interactable {
    pub kind: StationKind,
    pub radius: f32,
}

impl Interactable {
    pub fn crane() -> Self {
        Self {
            kind: StationKind::CraneConsole,
            radius: INTERACT_RADIUS,
        }
    }

    pub fn breaker(index: u8) -> Self {
        Self {
            kind: StationKind::PowerHourBreaker { index },
            radius: INTERACT_RADIUS,
        }
    }

    pub fn vault_objective() -> Self {
        Self {
            kind: StationKind::VaultObjective,
            radius: INTERACT_RADIUS,
        }
    }

    pub fn sort_chute(chute: u8) -> Self {
        Self {
            kind: StationKind::SortChute { chute },
            radius: INTERACT_RADIUS,
        }
    }

    pub fn coolant_valve(index: u8) -> Self {
        Self {
            kind: StationKind::CoolantValve { index },
            radius: INTERACT_RADIUS,
        }
    }

    pub fn meltdown_door(index: u8) -> Self {
        Self {
            kind: StationKind::MeltdownDoor { index },
            radius: INTERACT_RADIUS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationKind {
    CraneConsole,
    PowerHourBreaker { index: u8 },
    VaultObjective,
    SortChute { chute: u8 },
    CoolantValve { index: u8 },
    MeltdownDoor { index: u8 },
}

/// Client → server interact. Uses stable layout marker ids (not entity ids) so
/// clients and host resolve the same station after local room spawns.
#[derive(Event, Serialize, Deserialize, Clone, Debug)]
pub struct InteractRequest {
    pub marker_id: String,
}

#[derive(Resource, Default)]
pub struct InteractPrompt {
    pub message: String,
    pub last_action: String,
}

struct InteractOutcome {
    message: String,
    motion_success: bool,
    /// Set held freight kind after pickup.
    pick_up: Option<u8>,
    /// Clear held freight after sort / drop.
    clear_carry: bool,
    juice: crate::juice::JuiceEvent,
}

struct TournamentView {
    phase: TournamentPhase,
    room: RoomId,
    sort_target: u8,
}

fn read_tournament_view(
    director: &TournamentDirector,
    room: &RoomRuntime,
    snapshot: Option<&TournamentSnapshot>,
) -> TournamentView {
    if let Some(snap) = snapshot {
        TournamentView {
            phase: snap.phase,
            room: snap.room,
            sort_target: snap.sort_target,
        }
    } else {
        TournamentView {
            phase: director.phase,
            room: director.room,
            sort_target: room.sort_target,
        }
    }
}

fn handle_drop_freight(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut local_player: Query<
        (Entity, &Transform, Option<&CarryingFreight>, Option<&NetworkPlayer>),
        With<LocalPlayer>,
    >,
    mut prompt: ResMut<InteractPrompt>,
    mut juice: JuiceOut,
    director: Res<TournamentDirector>,
    mut scoring: ResMut<ScoringService>,
    config: Res<TournamentConfig>,
) {
    if !keyboard.just_pressed(KeyCode::KeyG) {
        return;
    }
    let Ok((player_entity, transform, carrying, network_player)) = local_player.single_mut() else {
        return;
    };
    let Some(held) = carrying.copied() else {
        return;
    };
    commands.entity(player_entity).remove::<CarryingFreight>();
    crate::player::freight::spawn_dropped_freight(
        &mut commands,
        &mut meshes,
        &mut materials,
        transform.translation,
        held.kind,
    );

    let mut msg = format!("Dropped {}.", held.label());
    if director.room == RoomId::CargoGantry
        && matches!(director.phase, TournamentPhase::RoomActive)
    {
        let player_id = match network_player {
            Some(p) => {
                let team = crate::tournament::types::bracket_team_index(p.slot, config.slot_size);
                let seat = crate::tournament::types::seat_in_team(p.slot, config.slot_size);
                PlayerScoreId(team * 10 + seat)
            }
            None => PlayerScoreId(config.human_slot.0 * 10),
        };
        scoring.record(player_id, ScoreAction::DroppedCarryable);
        msg = format!("Dropped crate — catch penalty! ({})", held.label());
    }
    prompt.last_action = msg;
    juice.play(JuiceEvent::GenericBad);
}

fn handle_world_freight_pickup(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut commands: Commands,
    local_player: Query<(Entity, &Transform, Option<&CarryingFreight>), With<LocalPlayer>>,
    freight: Query<(Entity, &Transform, &crate::player::WorldFreight)>,
    mut prompt: ResMut<InteractPrompt>,
    mut juice: JuiceOut,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok((player_entity, transform, carrying)) = local_player.single() else {
        return;
    };
    if carrying.is_some() {
        return;
    }
    let Some((entity, world_freight)) =
        crate::player::freight::nearest_world_freight(transform.translation, &freight, 2.2)
    else {
        return;
    };
    commands.entity(entity).despawn();
    commands.entity(player_entity).insert(CarryingFreight {
        kind: world_freight.kind,
    });
    prompt.last_action = format!(
        "Picked up {} — sort into the matching chute (F)",
        CarryingFreight {
            kind: world_freight.kind
        }
        .label()
    );
    juice.play(JuiceEvent::Pickup);
    // Consume F so station interact does not also fire this frame.
    keyboard.clear_just_pressed(KeyCode::KeyF);
}

fn handle_local_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    cli: Res<Cli>,
    config: Res<TournamentConfig>,
    director: Res<TournamentDirector>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    snapshots: Query<&TournamentSnapshot>,
    net: NetPresence,
    local_player: Query<
        (Entity, &Transform, Has<Leaseholder>, Option<&NetworkPlayer>, Option<&CarryingFreight>),
        With<LocalPlayer>,
    >,
    stations: Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
    mut jobs: Option<ResMut<JobSystem>>,
    mut boards: Query<(&mut JobBoard, &mut SmokeJobFlags)>,
    mut prompt: ResMut<InteractPrompt>,
    motion_anchors: Query<&InteractMotion>,
    mut juice: JuiceOut,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    let Ok((player_entity, player_transform, is_leaseholder, network_player, carrying)) =
        local_player.single()
    else {
        return;
    };

    if is_leaseholder {
        prompt.last_action = "Leaseholder: no hands — press V to ping the next panel.".into();
        return;
    }

    let Some((station_entity, station_transform, interactable, marker_id)) =
        nearest_interactable(player_transform.translation, &stations)
    else {
        return;
    };

    let view = read_tournament_view(
        director.as_ref(),
        room.as_ref(),
        snapshots.iter().next(),
    );
    let tournament_active =
        matches!(view.phase, TournamentPhase::RoomActive | TournamentPhase::Finale);

    let authority = net.server.is_some() || net.client.is_none();
    let marker_key = marker_id
        .map(|id| id.0.clone())
        .unwrap_or_else(|| format!("entity:{station_entity:?}"));

    if tournament_active {
        if cli.is_online() && !authority {
            let motion_success = predict_motion_success(interactable.kind, &view, room.as_ref());
            trigger_interact_motion(
                &mut commands,
                station_entity,
                interactable.kind,
                motion_success,
                &motion_anchors,
            );
            prompt.last_action = format!("Sent interact to `{marker_key}`");
            commands.client_trigger(InteractRequest {
                marker_id: marker_key,
            });
            return;
        }

        let (slot_id, player_id) = match network_player {
            Some(player) => {
                let team = crate::tournament::types::bracket_team_index(
                    player.slot,
                    config.slot_size,
                );
                let seat =
                    crate::tournament::types::seat_in_team(player.slot, config.slot_size);
                (SlotId(team), PlayerScoreId(team * 10 + seat))
            }
            None => (
                config.human_slot.clone(),
                PlayerScoreId(config.human_slot.0 * 10),
            ),
        };
        let outcome = apply_tournament_interact(
            interactable.kind,
            view.room,
            &slot_id,
            player_id,
            room.as_mut(),
            scoring.as_mut(),
            jobs.as_deref_mut(),
            &mut boards,
            carrying.copied(),
            &marker_key,
        );
        apply_carry_outcome(&mut commands, player_entity, &outcome);
        juice.play_at_station(
            outcome.juice,
            player_entity,
            station_transform.translation,
            player_transform.translation,
        );
        trigger_interact_motion(
            &mut commands,
            station_entity,
            interactable.kind,
            outcome.motion_success,
            &motion_anchors,
        );
        prompt.last_action = outcome.message;
        return;
    }

    // Lobby / between rooms: clients RPC; host + offline apply jobs locally.
    if cli.is_online() && !authority {
        trigger_interact_motion(
            &mut commands,
            station_entity,
            interactable.kind,
            true,
            &motion_anchors,
        );
        prompt.last_action = format!("Sent interact to `{marker_key}`");
        commands.client_trigger(InteractRequest {
            marker_id: marker_key,
        });
    } else if let Some(jobs) = jobs.as_deref_mut() {
        let result = apply_station_interact(jobs, interactable.kind);
        let motion_success = !result.contains("zapped") && !result.contains("wrong");
        trigger_interact_motion(
            &mut commands,
            station_entity,
            interactable.kind,
            motion_success,
            &motion_anchors,
        );
        prompt.last_action = format_result(interactable.kind, result);
        apply_job_action(jobs, &mut boards);
        let event = if motion_success {
            JuiceEvent::GenericOk
        } else {
            JuiceEvent::GenericBad
        };
        juice.play_at_station(
            event,
            player_entity,
            station_transform.translation,
            player_transform.translation,
        );
    }
}

fn apply_carry_outcome(commands: &mut Commands, player: Entity, outcome: &InteractOutcome) {
    if outcome.clear_carry {
        commands.entity(player).remove::<CarryingFreight>();
    }
    if let Some(kind) = outcome.pick_up {
        commands.entity(player).insert(CarryingFreight { kind });
    }
}

fn handle_interact_request(
    request: On<FromClient<InteractRequest>>,
    mut commands: Commands,
    director: Res<TournamentDirector>,
    config: Res<TournamentConfig>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    mut jobs: Option<ResMut<JobSystem>>,
    owners: Query<&crate::network::OwnedPlayer>,
    players: Query<
        (Entity, &Transform, &NetworkPlayer, Option<&CarryingFreight>),
        With<crate::player::NetworkPlayer>,
    >,
    stations: Query<(Entity, &Transform, &Interactable, &LayoutMarkerId)>,
    mut boards: Query<(&mut JobBoard, &mut SmokeJobFlags)>,
    motion_anchors: Query<&InteractMotion>,
    mut juice: JuiceOut,
) {
    let Some(client_entity) = request.client_id.entity() else {
        return;
    };

    let Ok(owned) = owners.get(client_entity) else {
        warn!("interact from unknown client `{client_entity}`");
        return;
    };

    let Ok((player_entity, player_transform, network_player, carrying)) = players.get(owned.0)
    else {
        return;
    };

    let Some((station_entity, station_transform, interactable, _)) = stations
        .iter()
        .find(|(_, _, _, marker)| marker.0 == request.marker_id)
    else {
        warn!("interact for unknown marker `{}`", request.marker_id);
        return;
    };

    if player_transform
        .translation
        .distance(station_transform.translation)
        > interactable.radius
    {
        return;
    }

    let tournament_active = matches!(
        director.phase,
        TournamentPhase::RoomActive | TournamentPhase::Finale
    );

    if tournament_active {
        let team = crate::tournament::types::bracket_team_index(
            network_player.slot,
            config.slot_size,
        );
        let seat =
            crate::tournament::types::seat_in_team(network_player.slot, config.slot_size);
        let slot_id = SlotId(team);
        let player_id = PlayerScoreId(team * 10 + seat);
        let outcome = apply_tournament_interact(
            interactable.kind,
            director.room,
            &slot_id,
            player_id,
            room.as_mut(),
            scoring.as_mut(),
            jobs.as_deref_mut(),
            &mut boards,
            carrying.copied(),
            &request.marker_id,
        );
        apply_carry_outcome(&mut commands, player_entity, &outcome);
        juice.play(outcome.juice);
        trigger_interact_motion(
            &mut commands,
            station_entity,
            interactable.kind,
            outcome.motion_success,
            &motion_anchors,
        );
        info!(
            "client `{client_entity}` tournament interact {:?}: {}",
            interactable.kind, outcome.message
        );
        return;
    }

    let Some(jobs) = jobs.as_deref_mut() else {
        return;
    };

    let result = apply_station_interact(jobs, interactable.kind);
    let motion_success = !result.contains("zapped") && !result.contains("wrong");
    trigger_interact_motion(
        &mut commands,
        station_entity,
        interactable.kind,
        motion_success,
        &motion_anchors,
    );
    apply_job_action(jobs, &mut boards);
    juice.play(if motion_success {
        JuiceEvent::GenericOk
    } else {
        JuiceEvent::GenericBad
    });
    info!(
        "client `{client_entity}` interacted with {:?}: {}",
        interactable.kind,
        format_result(interactable.kind, result)
    );
}

fn apply_tournament_interact(
    kind: StationKind,
    room_id: RoomId,
    slot_id: &SlotId,
    player_id: PlayerScoreId,
    room: &mut RoomRuntime,
    scoring: &mut ScoringService,
    jobs: Option<&mut JobSystem>,
    boards: &mut Query<(&mut JobBoard, &mut SmokeJobFlags)>,
    carrying: Option<CarryingFreight>,
    marker_id: &str,
) -> InteractOutcome {
    match kind {
        StationKind::SortChute { chute } => {
            // HR Orientation: must be carrying matching freight.
            if room_id == RoomId::HrOrientation {
                let Some(held) = carrying else {
                    return InteractOutcome {
                        message: format!(
                            "Pick up freight first (want: {})",
                            room.sort_target_label()
                        ),
                        motion_success: false,
                        pick_up: None,
                        clear_carry: false,
                        juice: JuiceEvent::GenericBad,
                    };
                };
                let success = held.kind == chute && chute == room.sort_target;
                if success {
                    room.sort_target = (room.sort_target + 1) % 4;
                    room.player_action(scoring, player_id, slot_id, ScoreAction::CorrectSort);
                    return InteractOutcome {
                        message: format!(
                            "Sorted {}! Next: {} · {}%",
                            held.label(),
                            room.sort_target_label(),
                            room.progress_percent()
                        ),
                        motion_success: true,
                        pick_up: None,
                        clear_carry: true,
                        juice: JuiceEvent::SortOk,
                    };
                }
                room.player_action(scoring, player_id, slot_id, ScoreAction::IncorrectSort);
                return InteractOutcome {
                    message: format!(
                        "Wrong chute! Dropped {}. Want: {}",
                        held.label(),
                        room.sort_target_label()
                    ),
                    motion_success: false,
                    pick_up: None,
                    clear_carry: true,
                    juice: JuiceEvent::SortBad,
                };
            }

            // Other rooms: legacy instant sort.
            let success = chute == room.sort_target;
            let action = if success {
                room.sort_target = (room.sort_target + 1) % 4;
                ScoreAction::CorrectSort
            } else {
                ScoreAction::IncorrectSort
            };
            room.player_action(scoring, player_id, slot_id, action);
            InteractOutcome {
                message: format!(
                    "Sorted! Vault progress: {}% (next: {})",
                    room.progress_percent(),
                    RoomRuntime::sort_label(room.sort_target)
                ),
                motion_success: success,
                pick_up: None,
                clear_carry: false,
                juice: if success {
                    JuiceEvent::SortOk
                } else {
                    JuiceEvent::SortBad
                },
            }
        }
        StationKind::VaultObjective => {
            if room_id == RoomId::HrOrientation {
                if carrying.is_some() {
                    return InteractOutcome {
                        message: format!(
                            "Already carrying {} — sort it into a chute (F)",
                            carrying.unwrap().label()
                        ),
                        motion_success: false,
                        pick_up: None,
                        clear_carry: false,
                        juice: JuiceEvent::GenericBad,
                    };
                }
                let kind = room.sort_target;
                return InteractOutcome {
                    message: format!(
                        "Picked up {} — sort into the matching chute (F)",
                        RoomRuntime::sort_label(kind)
                    ),
                    motion_success: true,
                    pick_up: Some(kind),
                    clear_carry: false,
                    juice: JuiceEvent::Pickup,
                };
            }

            // Cargo Gantry: pick up at crates/hook, deliver on the green pad.
            if room_id == RoomId::CargoGantry {
                let is_delivery = marker_id.contains("delivery");
                if is_delivery {
                    if carrying.is_none() {
                        return InteractOutcome {
                            message: "Bring a crate to the delivery pad (F)".into(),
                            motion_success: false,
                            pick_up: None,
                            clear_carry: false,
                            juice: JuiceEvent::GenericBad,
                        };
                    }
                    room.player_action(
                        scoring,
                        player_id,
                        slot_id,
                        ScoreAction::CrateDelivered,
                    );
                    return InteractOutcome {
                        message: format!(
                            "Delivered crate to the pad! {}%",
                            room.progress_percent()
                        ),
                        motion_success: true,
                        pick_up: None,
                        clear_carry: true,
                        juice: JuiceEvent::SortOk,
                    };
                }
                if carrying.is_some() {
                    return InteractOutcome {
                        message: "Already carrying — deliver to the green pad (F)".into(),
                        motion_success: false,
                        pick_up: None,
                        clear_carry: false,
                        juice: JuiceEvent::GenericBad,
                    };
                }
                return InteractOutcome {
                    message: "Picked up crate — haul it to the delivery pad (F)".into(),
                    motion_success: true,
                    pick_up: Some(0),
                    clear_carry: false,
                    juice: JuiceEvent::Pickup,
                };
            }

            let action = if room_id == RoomId::ShuttleMeltdown {
                ScoreAction::EscapeCrate
            } else {
                ScoreAction::CrateDelivered
            };
            room.player_action(scoring, player_id, slot_id, action);
            InteractOutcome {
                message: format!("Vault progress: {}%", room.progress_percent()),
                motion_success: true,
                pick_up: None,
                clear_carry: false,
                juice: JuiceEvent::GenericOk,
            }
        }
        StationKind::CoolantValve { .. } => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::CoolantValve);
            InteractOutcome {
                message: format!(
                    "Coolant flooded — meltdown down to {}%",
                    room.meltdown_percent()
                ),
                motion_success: true,
                pick_up: None,
                clear_carry: false,
                juice: JuiceEvent::GenericOk,
            }
        }
        StationKind::MeltdownDoor { .. } => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::DoorSealed);
            InteractOutcome {
                message: format!("Bay door SEALED — progress {}%", room.progress_percent()),
                motion_success: true,
                pick_up: None,
                clear_carry: false,
                juice: JuiceEvent::SortOk,
            }
        }
        StationKind::CraneConsole => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::CrateDelivered);
            let message = if let Some(jobs) = jobs {
                let msg = apply_station_interact(jobs, kind);
                apply_job_action(jobs, boards);
                format!("Gantry THUD — {msg} · {}%", room.progress_percent())
            } else {
                format!("Gantry delivery — {}%", room.progress_percent())
            };
            InteractOutcome {
                message,
                motion_success: true,
                pick_up: None,
                clear_carry: false,
                juice: JuiceEvent::SortOk,
            }
        }
        StationKind::PowerHourBreaker { index } => {
            if let Some(jobs) = jobs {
                let msg = apply_station_interact(jobs, kind);
                apply_job_action(jobs, boards);
                let success = !msg.contains("zapped");
                let action = if success {
                    ScoreAction::BreakerCorrect
                } else {
                    ScoreAction::BreakerWrong
                };
                room.player_action(scoring, player_id, slot_id, action);
                let step = jobs.power_hour_step();
                InteractOutcome {
                    message: if success {
                        format!("Breaker {} OK — sequence step {}", index + 1, step + 1)
                    } else {
                        format!(
                            "ZAP! Breaker {} wrong — want order 1→3→2→4 (step {})",
                            index + 1,
                            step + 1
                        )
                    },
                    motion_success: success,
                    pick_up: None,
                    clear_carry: false,
                    juice: if success {
                        JuiceEvent::GenericOk
                    } else {
                        JuiceEvent::Knockback
                    },
                }
            } else {
                room.player_action(scoring, player_id, slot_id, ScoreAction::BreakerCorrect);
                InteractOutcome {
                    message: format!("Breaker {index} flipped"),
                    motion_success: true,
                    pick_up: None,
                    clear_carry: false,
                    juice: JuiceEvent::GenericOk,
                }
            }
        }
    }
}

fn predict_motion_success(kind: StationKind, view: &TournamentView, _room: &RoomRuntime) -> bool {
    match kind {
        StationKind::SortChute { chute } => chute == view.sort_target,
        StationKind::PowerHourBreaker { .. } => true,
        _ => true,
    }
}

fn apply_station_interact(jobs: &mut JobSystem, kind: StationKind) -> String {
    match kind {
        StationKind::CraneConsole => match jobs.try_crane_interact() {
            JobActionResult::Progressed => "crane advanced".into(),
            JobActionResult::Completed => "crane complete".into(),
            JobActionResult::AlreadyComplete => "crane already done".into(),
            JobActionResult::WrongSequence => "crane wrong sequence".into(),
            JobActionResult::NotActive => "crane inactive".into(),
            JobActionResult::Ignored => "crane ignored".into(),
        },
        StationKind::PowerHourBreaker { index } => {
            let label = breaker_panel_label(index);
            match jobs.try_power_hour_interact(index) {
                BreakerResult::Flipped => format!("{label} flipped"),
                BreakerResult::Completed => "power hour complete".into(),
                BreakerResult::WrongBreaker => format!("{label} zapped (wrong order)"),
            }
        }
        StationKind::VaultObjective => "vault objective advanced".into(),
        StationKind::SortChute { .. } => "sort chute".into(),
        StationKind::CoolantValve { index } => format!("coolant valve {index}"),
        StationKind::MeltdownDoor { index } => format!("sealed door {index}"),
    }
}

fn format_result(kind: StationKind, message: String) -> String {
    match kind {
        StationKind::CraneConsole => format!("Crane: {message}"),
        StationKind::PowerHourBreaker { index } => format!("Breaker {index}: {message}"),
        StationKind::VaultObjective => format!("Vault: {message}"),
        StationKind::SortChute { chute } => format!("Chute {chute}: {message}"),
        StationKind::CoolantValve { index } => format!("Coolant {index}: {message}"),
        StationKind::MeltdownDoor { index } => format!("Door {index}: {message}"),
    }
}

fn show_interact_prompt(
    director: Res<TournamentDirector>,
    room: Res<RoomRuntime>,
    snapshots: Query<&TournamentSnapshot>,
    local_player: Query<(&Transform, Option<&CarryingFreight>, Has<Leaseholder>), With<LocalPlayer>>,
    stations: Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
    jobs: Option<Res<JobSystem>>,
    boards: Query<&JobBoard>,
    mut prompt: ResMut<InteractPrompt>,
) {
    let Ok((player_transform, carrying, is_leaseholder)) = local_player.single() else {
        prompt.message.clear();
        return;
    };

    let Some((_, _, interactable, _)) =
        nearest_interactable(player_transform.translation, &stations)
    else {
        prompt.message.clear();
        return;
    };

    if is_leaseholder {
        prompt.message = match interactable.kind {
            StationKind::PowerHourBreaker { index } => {
                let label = breaker_panel_label(index);
                let step = jobs.map(|j| j.power_hour_step()).unwrap_or(0) as usize;
                let next = crate::core::POWER_HOUR_SEQUENCE
                    .get(step)
                    .copied()
                    .unwrap_or(0);
                format!(
                    "Leaseholder — next flip: {} · looking at {label} · V to ping",
                    breaker_panel_label(next)
                )
            }
            _ => "Leaseholder — press V to ping (no hands)".into(),
        };
        return;
    }

    let view = read_tournament_view(
        director.as_ref(),
        room.as_ref(),
        snapshots.iter().next(),
    );

    prompt.message = prompt_for_station(
        interactable,
        jobs.as_deref(),
        boards.iter().next(),
        &view,
        carrying.copied(),
    );
}

fn prompt_for_station(
    interactable: &Interactable,
    jobs: Option<&JobSystem>,
    board: Option<&JobBoard>,
    view: &TournamentView,
    carrying: Option<CarryingFreight>,
) -> String {
    match interactable.kind {
        StationKind::CraneConsole => {
            let (current, target) = jobs
                .map(|jobs| jobs.progress_for(CRANE_JOB_ID))
                .unwrap_or((0, 3));
            if jobs.is_some_and(|jobs| jobs.is_complete(CRANE_JOB_ID))
                || board.is_some_and(|board| {
                    board
                        .states
                        .get(CRANE_JOB_ID)
                        .is_some_and(|state| state.complete)
                })
            {
                "Crane of Regret — complete".into()
            } else if jobs.is_some_and(|jobs| jobs.is_active(CRANE_JOB_ID)) {
                format!("Press F — Crane of Regret ({current}/{target})")
            } else {
                "Press F — Start Crane of Regret".into()
            }
        }
        StationKind::PowerHourBreaker { index } => {
            let label = breaker_panel_label(index);
            if jobs.is_some_and(|jobs| jobs.is_complete(POWER_HOUR_JOB_ID)) {
                return format!("{label} — power restored");
            }
            if jobs.is_some_and(|jobs| !jobs.is_active(POWER_HOUR_JOB_ID)) {
                return format!("Press F — Start Power Hour ({label})");
            }
            let step = jobs.map(|jobs| jobs.power_hour_step()).unwrap_or(0);
            format!("Press F — Flip {label} (step {})", step + 1)
        }
        StationKind::VaultObjective => {
            if view.room == RoomId::HrOrientation {
                if let Some(held) = carrying {
                    format!(
                        "Carrying {} — find the matching chute (G to drop)",
                        held.label()
                    )
                } else {
                    format!(
                        "Press F — pick up {} freight",
                        RoomRuntime::sort_label(view.sort_target)
                    )
                }
            } else if view.room == RoomId::CargoGantry {
                if carrying.is_some() {
                    "Press F — deliver crate (or haul to green pad)".into()
                } else {
                    "Press F — pick up crate for delivery".into()
                }
            } else if view.room == RoomId::ShuttleMeltdown {
                "Press F — load escape crate".into()
            } else {
                "Press F — deliver crate".into()
            }
        }
        StationKind::SortChute { chute } => {
            if view.room == RoomId::HrOrientation {
                match carrying {
                    Some(held) if held.kind == chute && chute == view.sort_target => {
                        format!("Press F — sort {} here", held.label())
                    }
                    Some(held) => format!(
                        "Press F — dump {} into chute {}? (want: {})",
                        held.label(),
                        chute + 1,
                        RoomRuntime::sort_label(view.sort_target)
                    ),
                    None => format!(
                        "Chute {} wants {} — pick up freight first",
                        chute + 1,
                        RoomRuntime::sort_label(view.sort_target)
                    ),
                }
            } else {
                format!(
                    "Press F — sort into chute {} (want: {})",
                    chute + 1,
                    RoomRuntime::sort_label(view.sort_target)
                )
            }
        }
        StationKind::CoolantValve { index } => {
            format!("Press F — turn coolant valve {}", index + 1)
        }
        StationKind::MeltdownDoor { index } => {
            format!("Press F — seal door {}", index + 1)
        }
    }
}

pub fn nearest_interactable<'a>(
    player: Vec3,
    stations: &'a Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
) -> Option<(Entity, &'a Transform, &'a Interactable, Option<&'a LayoutMarkerId>)> {
    stations
        .iter()
        .filter(|(_, transform, interactable, _)| {
            player.distance(transform.translation) <= interactable.radius
        })
        .min_by(|(_, left, _, _), (_, right, _, _)| {
            left.translation
                .distance(player)
                .partial_cmp(&right.translation.distance(player))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Headless smoke helper — interact with nearest station without input.
pub fn auto_interact_nearest(
    player_pos: Vec3,
    stations: &Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
    jobs: &mut JobSystem,
    boards: &mut Query<'_, '_, (&mut JobBoard, &mut SmokeJobFlags)>,
) -> bool {
    let Some((_, _, interactable, _)) = nearest_interactable(player_pos, stations) else {
        return false;
    };
    apply_station_interact(jobs, interactable.kind);
    apply_job_action(jobs, boards);
    true
}
