//! World pings for info asymmetry (breaker labels, leaseholder callouts).

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    flow::AppScreen,
    interaction::{nearest_interactable, Interactable, StationKind},
    jobs::JobSystem,
    player::{Leaseholder, LocalPlayer},
    rooms::LayoutMarkerId,
    scoring::{PlayerScoreId, ScoreAction, ScoringService},
    tournament::{TournamentConfig, TournamentDirector, TournamentPhase},
    world::GameplayEntity,
    Cli,
};

/// Cryptic / misleading panel names — workers see these; leaseholder sees sequence truth.
pub fn breaker_panel_label(index: u8) -> &'static str {
    match index {
        0 => "COFFEE",
        1 => "PRINTERS",
        2 => "TOASTERS",
        3 => "AIRLOCK",
        4 => "HR",
        5 => "VENDING",
        6 => "GRAVITY",
        7 => "SNACKS",
        8 => "LEGAL",
        9 => "WIFI",
        10 => "TRASH",
        11 => "ESCAPE",
        _ => "UNKNOWN",
    }
}

#[derive(Component)]
pub struct PingBeacon {
    pub ttl: f32,
}

#[derive(Resource, Debug, Default)]
pub struct PingFeed {
    pub last_line: String,
    pub timer: f32,
}

impl PingFeed {
    pub fn push(&mut self, line: impl Into<String>) {
        self.last_line = line.into();
        self.timer = 4.0;
    }
}

#[derive(Event, Serialize, Deserialize, Clone, Debug)]
pub struct PingRequest {
    pub marker_id: String,
}

pub struct PingPlugin;

impl Plugin for PingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PingFeed>()
            .add_client_event::<PingRequest>(Channel::Unordered)
            .add_observer(handle_ping_request)
            .add_systems(
                Update,
                (
                    handle_local_ping,
                    tick_ping_beacons,
                    tick_ping_feed,
                )
                    .run_if(in_state(AppScreen::Playing)),
            );
    }
}

fn handle_local_ping(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    cli: Res<Cli>,
    config: Res<TournamentConfig>,
    director: Res<TournamentDirector>,
    mut feed: ResMut<PingFeed>,
    mut scoring: ResMut<ScoringService>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local: Query<
        (&Transform, Has<Leaseholder>, Option<&crate::player::NetworkPlayer>),
        With<LocalPlayer>,
    >,
    stations: Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
    jobs: Option<Res<JobSystem>>,
    client: Option<Res<bevy_replicon_renet::RenetClient>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyV) {
        return;
    }

    let Ok((transform, is_leaseholder, network_player)) = local.single() else {
        return;
    };

    let Some((entity, station_tf, interactable, marker)) =
        nearest_interactable(transform.translation, &stations)
    else {
        feed.push("Ping: nothing in range");
        return;
    };

    let marker_id = marker
        .map(|m| m.0.clone())
        .unwrap_or_else(|| format!("entity:{entity:?}"));

    let line = format_ping_line(
        interactable.kind,
        is_leaseholder,
        jobs.as_deref(),
        director.phase,
    );

    // Clients forward to host; host/offline apply locally.
    if cli.is_online() && client.is_some() {
        commands.client_trigger(PingRequest {
            marker_id: marker_id.clone(),
        });
        feed.push(format!("Ping sent — {line}"));
    } else {
        feed.push(line.clone());
        spawn_beacon(
            &mut commands,
            &mut meshes,
            &mut materials,
            station_tf.translation,
        );
        let player_id = match network_player {
            Some(p) => {
                let team = crate::tournament::types::bracket_team_index(p.slot, config.slot_size);
                let seat = crate::tournament::types::seat_in_team(p.slot, config.slot_size);
                PlayerScoreId(team * 10 + seat)
            }
            None => PlayerScoreId(config.human_slot.0 * 10),
        };
        scoring.record(player_id, ScoreAction::CorrectPing);
    }
}

fn handle_ping_request(
    request: On<FromClient<PingRequest>>,
    mut commands: Commands,
    mut feed: ResMut<PingFeed>,
    mut scoring: ResMut<ScoringService>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    owners: Query<&crate::network::OwnedPlayer>,
    players: Query<(&crate::player::NetworkPlayer, Has<Leaseholder>)>,
    stations: Query<(&Transform, &Interactable, &LayoutMarkerId)>,
    jobs: Option<Res<JobSystem>>,
    director: Res<TournamentDirector>,
    mut announcer: ResMut<crate::announcer::AnnouncerQueue>,
) {
    let Some(client_entity) = request.client_id.entity() else {
        return;
    };
    let Ok(owned) = owners.get(client_entity) else {
        return;
    };
    let Ok((net, is_leaseholder)) = players.get(owned.0) else {
        return;
    };

    let Some((tf, interactable, _)) = stations
        .iter()
        .find(|(_, _, m)| m.0 == request.marker_id)
    else {
        return;
    };

    let line = format_ping_line(
        interactable.kind,
        is_leaseholder,
        jobs.as_deref(),
        director.phase,
    );
    feed.push(format!("P{}: {line}", net.slot));
    announcer.push(format!("Ping — {line}"));
    spawn_beacon(&mut commands, &mut meshes, &mut materials, tf.translation);
    let team = crate::tournament::types::bracket_team_index(net.slot, director.slot_size());
    let seat = crate::tournament::types::seat_in_team(net.slot, director.slot_size());
    scoring.record(PlayerScoreId(team * 10 + seat), ScoreAction::CorrectPing);
}

fn format_ping_line(
    kind: StationKind,
    is_leaseholder: bool,
    jobs: Option<&JobSystem>,
    phase: TournamentPhase,
) -> String {
    match kind {
        StationKind::PowerHourBreaker { index } => {
            let label = breaker_panel_label(index);
            if is_leaseholder
                && matches!(phase, TournamentPhase::RoomActive | TournamentPhase::Finale)
            {
                let step = jobs.map(|j| j.power_hour_step()).unwrap_or(0) as usize;
                let seq = crate::core::POWER_HOUR_SEQUENCE;
                let next = seq.get(step).copied().unwrap_or(0);
                let next_label = breaker_panel_label(next);
                format!("LEASE: flip {next_label} next (panel looks like {label})")
            } else {
                format!("Panel marked {label} — is this the one?")
            }
        }
        StationKind::SortChute { chute } => {
            format!("Ping: chute {} ({})", chute + 1, crate::rooms::RoomRuntime::sort_label(chute))
        }
        StationKind::VaultObjective => "Ping: objective here".into(),
        StationKind::CraneConsole => "Ping: crane console".into(),
        StationKind::CoolantValve { index } => format!("Ping: coolant valve {}", index + 1),
        StationKind::MeltdownDoor { index } => format!("Ping: seal door {}", index + 1),
    }
}

fn spawn_beacon(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    at: Vec3,
) {
    commands.spawn((
        PingBeacon { ttl: 3.0 },
        GameplayEntity,
        Mesh3d(meshes.add(Sphere::new(0.22))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.85, 0.2),
            emissive: LinearRgba::rgb(2.0, 1.4, 0.2),
            unlit: true,
            ..Default::default()
        })),
        Transform::from_translation(at + Vec3::Y * 2.2),
        Name::new("PingBeacon"),
    ));
}

fn tick_ping_beacons(
    time: Res<Time>,
    mut commands: Commands,
    mut beacons: Query<(Entity, &mut PingBeacon, &mut Transform)>,
) {
    for (entity, mut beacon, mut tf) in &mut beacons {
        beacon.ttl -= time.delta_secs();
        tf.translation.y += time.delta_secs() * 0.35;
        if beacon.ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_ping_feed(time: Res<Time>, mut feed: ResMut<PingFeed>) {
    if feed.timer > 0.0 {
        feed.timer -= time.delta_secs();
        if feed.timer <= 0.0 {
            feed.last_line.clear();
        }
    }
}
