use bevy::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};

use crate::{
    data::RoomLayoutCatalog,
    player::NetworkPlayer,
    rooms::spawner::{spawn_layout_marker, MarkerTag},
    tournament::{
        is_tournament_authority, types::RoomId, types::TournamentPhase, TournamentSnapshot,
    },
};

/// Despawned and rebuilt when the tournament enters a new vault stage.
#[derive(Component, Debug, Clone)]
pub struct RoomLayoutPiece;

/// Stable marker id from `data/rooms/*.json` — swap GLBs without moving gameplay slots.
#[derive(Component, Debug, Clone)]
pub struct LayoutMarkerId(pub String);

#[derive(Resource, Debug, Clone, Copy)]
pub struct RoomSpawnPoint {
    pub lobby: Vec3,
    pub current: Vec3,
}

impl Default for RoomSpawnPoint {
    fn default() -> Self {
        Self {
            lobby: Vec3::new(0.0, 1.0, 12.0),
            current: Vec3::new(0.0, 1.0, 12.0),
        }
    }
}

#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActiveRoomLayout {
    pub room: Option<RoomId>,
    pub phase: Option<TournamentPhase>,
}

pub fn sync_room_layout(
    director: Res<crate::tournament::TournamentDirector>,
    snapshots: Query<&TournamentSnapshot>,
    catalog: Res<RoomLayoutCatalog>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
    mut active: ResMut<ActiveRoomLayout>,
    mut spawn_point: ResMut<RoomSpawnPoint>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<crate::data::StudioRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pieces: Query<Entity, With<RoomLayoutPiece>>,
) {
    let authority = is_tournament_authority(server, client);
    let (phase, room) = if authority {
        (director.phase, director.room)
    } else if let Some(snap) = snapshots.iter().next() {
        (snap.phase, snap.room)
    } else {
        return;
    };

    let desired_room = match phase {
        TournamentPhase::RoomActive | TournamentPhase::Finale => Some(room),
        _ => None,
    };

    if active.room == desired_room && active.phase == Some(phase) {
        return;
    }

    for entity in &pieces {
        commands.entity(entity).despawn();
    }

    active.room = desired_room;
    active.phase = Some(phase);

    let Some(room) = desired_room else {
        spawn_point.current = spawn_point.lobby;
        return;
    };

    let Some(layout) = catalog.get(room) else {
        warn!("no room layout definition for {:?}", room);
        return;
    };

    spawn_point.current = Vec3::from_array(layout.player_spawn);

    for marker in &layout.markers {
        spawn_layout_marker(
            &mut commands,
            &asset_server,
            registry.as_ref(),
            &mut meshes,
            &mut materials,
            marker,
            MarkerTag::Room,
        );
    }

    info!(
        "room layout spawned: {} ({} markers, spawn {:?})",
        room.label(),
        layout.markers.len(),
        spawn_point.current
    );
}

/// Teleport players to the active room spawn when a vault stage loads.
pub fn relocate_players_on_room_enter(
    active: Res<ActiveRoomLayout>,
    spawn_point: Res<RoomSpawnPoint>,
    mut players: Query<&mut Transform, With<NetworkPlayer>>,
    mut last_room: Local<Option<RoomId>>,
) {
    let Some(room) = active.room else {
        *last_room = None;
        return;
    };

    if last_room.as_ref() == Some(&room) {
        return;
    }

    for mut transform in &mut players {
        transform.translation = spawn_point.current;
    }

    info!(
        "relocated players to {:?} for {}",
        spawn_point.current,
        room.label()
    );
    *last_room = Some(room);
}
