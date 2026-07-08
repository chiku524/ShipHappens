pub mod motion;

use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};
use serde::{Deserialize, Serialize};

use crate::{
    core::{CRANE_JOB_ID, INTERACT_RADIUS, POWER_HOUR_JOB_ID},
    jobs::{apply_job_action, BreakerResult, JobActionResult, JobBoard, JobSystem, SmokeJobFlags},
    player::{Leaseholder, LocalPlayer, NetworkPlayer},
    rooms::RoomRuntime,
    scoring::{PlayerScoreId, ScoreAction, ScoringService},
    tournament::{
        is_tournament_authority, types::RoomId, types::SlotId, TournamentConfig, TournamentDirector,
        TournamentPhase, TournamentSnapshot,
    },
    Cli,
};

pub use motion::{attach_interact_motion, trigger_interact_motion, InteractMotion};

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractPrompt>()
            .add_mapped_client_event::<InteractRequest>(Channel::Ordered)
            .add_observer(handle_interact_request)
            .add_systems(
                Update,
                (
                    show_interact_prompt,
                    handle_local_interact,
                    motion::tick_interact_motion,
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

#[derive(Event, Serialize, Deserialize, Clone, Copy, Debug, MapEntities)]
pub struct InteractRequest {
    #[entities]
    pub station: Entity,
}

#[derive(Resource, Default)]
pub struct InteractPrompt {
    pub message: String,
    pub last_action: String,
}

struct InteractOutcome {
    message: String,
    motion_success: bool,
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

fn handle_local_interact(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    cli: Res<Cli>,
    config: Res<TournamentConfig>,
    director: Res<TournamentDirector>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    snapshots: Query<&TournamentSnapshot>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
    local_player: Query<
        (&Transform, Has<Leaseholder>, Option<&NetworkPlayer>),
        With<LocalPlayer>,
    >,
    stations: Query<(Entity, &Transform, &Interactable)>,
    mut jobs: Option<ResMut<JobSystem>>,
    mut boards: Query<(&mut JobBoard, &mut SmokeJobFlags)>,
    mut prompt: ResMut<InteractPrompt>,
    motion_anchors: Query<&InteractMotion>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }

    let Ok((player_transform, is_leaseholder, network_player)) = local_player.single() else {
        return;
    };

    if is_leaseholder {
        prompt.last_action = "Leaseholder: direct only — use pings.".into();
        return;
    }

    let Some((station_entity, _, interactable)) =
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

    if tournament_active {
        if cli.is_online() && !is_tournament_authority(server, client) {
            let motion_success = predict_motion_success(interactable.kind, &view, room.as_ref());
            trigger_interact_motion(
                &mut commands,
                station_entity,
                interactable.kind,
                motion_success,
                &motion_anchors,
            );
            prompt.last_action = format!("Sent interact to `{station_entity:?}`");
            commands.client_trigger(InteractRequest {
                station: station_entity,
            });
            return;
        }

        let slot_id = network_player
            .map(|player| SlotId(player.slot))
            .unwrap_or_else(|| config.human_slot.clone());
        let player_id = PlayerScoreId(slot_id.0 * 10);
        let outcome = apply_tournament_interact(
            interactable.kind,
            view.room,
            &slot_id,
            player_id,
            room.as_mut(),
            scoring.as_mut(),
            jobs.as_deref_mut(),
            &mut boards,
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

    if cli.is_online() {
        trigger_interact_motion(
            &mut commands,
            station_entity,
            interactable.kind,
            true,
            &motion_anchors,
        );
        prompt.last_action = format!("Sent interact to `{station_entity:?}`");
        commands.client_trigger(InteractRequest {
            station: station_entity,
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
    }
}

fn handle_interact_request(
    request: On<FromClient<InteractRequest>>,
    mut commands: Commands,
    director: Res<TournamentDirector>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    mut jobs: Option<ResMut<JobSystem>>,
    owners: Query<&crate::network::OwnedPlayer>,
    players: Query<(&Transform, &NetworkPlayer), With<crate::player::NetworkPlayer>>,
    stations: Query<(Entity, &Transform, &Interactable)>,
    mut boards: Query<(&mut JobBoard, &mut SmokeJobFlags)>,
    motion_anchors: Query<&InteractMotion>,
) {
    let Some(client_entity) = request.client_id.entity() else {
        return;
    };

    let Ok(owned) = owners.get(client_entity) else {
        warn!("interact from unknown client `{client_entity}`");
        return;
    };

    let Ok((player_transform, network_player)) = players.get(owned.0) else {
        return;
    };

    let Ok((_, station_transform, interactable)) = stations.get(request.station) else {
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
        let slot_id = SlotId(network_player.slot);
        let player_id = PlayerScoreId(slot_id.0 * 10);
        let outcome = apply_tournament_interact(
            interactable.kind,
            director.room,
            &slot_id,
            player_id,
            room.as_mut(),
            scoring.as_mut(),
            jobs.as_deref_mut(),
            &mut boards,
        );
        trigger_interact_motion(
            &mut commands,
            request.station,
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
        request.station,
        interactable.kind,
        motion_success,
        &motion_anchors,
    );
    apply_job_action(jobs, &mut boards);
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
) -> InteractOutcome {
    match kind {
        StationKind::SortChute { chute } => {
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
            }
        }
        StationKind::VaultObjective => {
            let action = if room_id == RoomId::ShuttleMeltdown {
                ScoreAction::EscapeCrate
            } else {
                ScoreAction::CrateDelivered
            };
            room.player_action(scoring, player_id, slot_id, action);
            InteractOutcome {
                message: format!("Vault progress: {}%", room.progress_percent()),
                motion_success: true,
            }
        }
        StationKind::CoolantValve { .. } => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::CoolantValve);
            InteractOutcome {
                message: format!(
                    "Coolant valve turned — meltdown {}%",
                    room.progress.meltdown_meter as u32
                ),
                motion_success: true,
            }
        }
        StationKind::MeltdownDoor { .. } => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::DoorSealed);
            InteractOutcome {
                message: format!("Door sealed — progress {}%", room.progress_percent()),
                motion_success: true,
            }
        }
        StationKind::CraneConsole => {
            room.player_action(scoring, player_id, slot_id, ScoreAction::CrateDelivered);
            let message = if let Some(jobs) = jobs {
                let msg = apply_station_interact(jobs, kind);
                apply_job_action(jobs, boards);
                format!("Gantry: {msg} | {}%", room.progress_percent())
            } else {
                format!("Gantry delivery — {}%", room.progress_percent())
            };
            InteractOutcome {
                message,
                motion_success: true,
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
                InteractOutcome {
                    message: format!("Breaker {index}: {msg}"),
                    motion_success: success,
                }
            } else {
                room.player_action(scoring, player_id, slot_id, ScoreAction::BreakerCorrect);
                InteractOutcome {
                    message: format!("Breaker {index} flipped"),
                    motion_success: true,
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
        StationKind::PowerHourBreaker { index } => match jobs.try_power_hour_interact(index) {
            BreakerResult::Flipped => format!("breaker {index} flipped"),
            BreakerResult::Completed => "power hour complete".into(),
            BreakerResult::WrongBreaker => format!("breaker {index} zapped (wrong order)"),
        },
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
    local_player: Query<&Transform, With<LocalPlayer>>,
    stations: Query<(Entity, &Transform, &Interactable)>,
    jobs: Option<Res<JobSystem>>,
    boards: Query<&JobBoard>,
    mut prompt: ResMut<InteractPrompt>,
) {
    let Ok(player_transform) = local_player.single() else {
        prompt.message.clear();
        return;
    };

    let Some((_, _, interactable)) =
        nearest_interactable(player_transform.translation, &stations)
    else {
        prompt.message.clear();
        return;
    };

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
    );
}

fn prompt_for_station(
    interactable: &Interactable,
    jobs: Option<&JobSystem>,
    board: Option<&JobBoard>,
    view: &TournamentView,
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
            if jobs.is_some_and(|jobs| jobs.is_complete(POWER_HOUR_JOB_ID)) {
                return format!("Breaker {} — power restored", index + 1);
            }
            if jobs.is_some_and(|jobs| !jobs.is_active(POWER_HOUR_JOB_ID)) {
                return format!("Press F — Start Power Hour (breaker {})", index + 1);
            }
            let step = jobs.map(|jobs| jobs.power_hour_step()).unwrap_or(0);
            format!(
                "Press F — Flip breaker {} (step {})",
                index + 1,
                step + 1
            )
        }
        StationKind::VaultObjective => {
            if view.room == RoomId::ShuttleMeltdown {
                "Press F — load escape crate".into()
            } else {
                "Press F — deliver crate".into()
            }
        }
        StationKind::SortChute { chute } => {
            format!(
                "Press F — sort into chute {} (want: {})",
                chute + 1,
                RoomRuntime::sort_label(view.sort_target)
            )
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
    stations: &'a Query<(Entity, &Transform, &Interactable)>,
) -> Option<(Entity, &'a Transform, &'a Interactable)> {
    stations
        .iter()
        .filter(|(_, transform, interactable)| {
            player.distance(transform.translation) <= interactable.radius
        })
        .min_by(|(_, left, _), (_, right, _)| {
            left.translation
                .distance(player)
                .partial_cmp(&right.translation.distance(player))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, transform, interactable)| (entity, transform, interactable))
}

/// Headless smoke helper — interact with nearest station without input.
pub fn auto_interact_nearest(
    player_pos: Vec3,
    stations: &Query<(Entity, &Transform, &Interactable)>,
    jobs: &mut JobSystem,
    boards: &mut Query<'_, '_, (&mut JobBoard, &mut SmokeJobFlags)>,
) -> bool {
    let Some((_, _, interactable)) = nearest_interactable(player_pos, stations) else {
        return false;
    };
    apply_station_interact(jobs, interactable.kind);
    apply_job_action(jobs, boards);
    true
}
