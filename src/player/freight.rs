//! World freight piles / conveyor cadence for HR Orientation.

use bevy::prelude::*;

use crate::{
    rooms::RoomRuntime,
    tournament::{types::RoomId, TournamentDirector, TournamentPhase},
    world::GameplayEntity,
};

/// Loose freight on the bay floor — walk up and press F to pick up.
#[derive(Component, Debug, Clone, Copy)]
pub struct WorldFreight {
    pub kind: u8,
}

#[derive(Resource, Debug)]
pub struct FreightCadence {
    pub spawn_timer: f32,
    pub next_kind: u8,
}

impl Default for FreightCadence {
    fn default() -> Self {
        Self {
            spawn_timer: 1.5,
            next_kind: 0,
        }
    }
}

const MAX_WORLD_FREIGHT: usize = 6;
const SPAWN_INTERVAL: f32 = 1.8;
const PILE_CENTER: Vec3 = Vec3::new(-4.0, 0.35, 5.0);

pub fn tick_freight_cadence(
    time: Res<Time>,
    director: Res<TournamentDirector>,
    room: Res<RoomRuntime>,
    mut cadence: ResMut<FreightCadence>,
    existing: Query<&WorldFreight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if director.phase != TournamentPhase::RoomActive || room.active != Some(RoomId::HrOrientation)
    {
        return;
    }
    if room.progress.cleared || room.failed {
        return;
    }

    cadence.spawn_timer -= time.delta_secs();
    if cadence.spawn_timer > 0.0 {
        return;
    }
    cadence.spawn_timer = SPAWN_INTERVAL;

    if existing.iter().count() >= MAX_WORLD_FREIGHT {
        return;
    }

    let kind = cadence.next_kind % 4;
    cadence.next_kind = cadence.next_kind.wrapping_add(1);

    let angle = (cadence.next_kind as f32) * 1.7;
    let offset = Vec3::new(angle.cos() * 1.6, 0.0, angle.sin() * 1.2);
    let color = freight_color(kind);

    commands.spawn((
        WorldFreight { kind },
        GameplayEntity,
        Mesh3d(meshes.add(Cuboid::new(0.55, 0.55, 0.55))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        })),
        Transform::from_translation(PILE_CENTER + offset),
        Name::new(format!("WorldFreight_{}", RoomRuntime::sort_label(kind))),
    ));
}

pub fn clear_world_freight_outside_hr(
    director: Res<TournamentDirector>,
    room: Res<RoomRuntime>,
    freight: Query<Entity, With<WorldFreight>>,
    mut commands: Commands,
    mut cadence: ResMut<FreightCadence>,
) {
    let hr_live = matches!(
        director.phase,
        TournamentPhase::RoomActive | TournamentPhase::Finale
    ) && room.active == Some(RoomId::HrOrientation);

    if hr_live {
        return;
    }

    let mut cleared = false;
    for entity in &freight {
        commands.entity(entity).despawn();
        cleared = true;
    }
    if cleared {
        *cadence = FreightCadence::default();
    }
}

pub fn nearest_world_freight<'a>(
    player: Vec3,
    freight: &'a Query<(Entity, &Transform, &WorldFreight)>,
    radius: f32,
) -> Option<(Entity, &'a WorldFreight)> {
    freight
        .iter()
        .filter(|(_, transform, _)| player.distance(transform.translation) <= radius)
        .min_by(|(_, a, _), (_, b, _)| {
            a.translation
                .distance(player)
                .partial_cmp(&b.translation.distance(player))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, _, freight)| (entity, freight))
}

pub fn spawn_dropped_freight(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    position: Vec3,
    kind: u8,
) {
    commands.spawn((
        WorldFreight { kind },
        GameplayEntity,
        Mesh3d(meshes.add(Cuboid::new(0.55, 0.55, 0.55))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: freight_color(kind),
            ..Default::default()
        })),
        Transform::from_translation(Vec3::new(position.x, 0.35, position.z)),
        Name::new(format!("Dropped_{}", RoomRuntime::sort_label(kind))),
    ));
}

fn freight_color(kind: u8) -> Color {
    match kind {
        0 => Color::srgb(0.95, 0.45, 0.15),
        1 => Color::srgb(0.75, 0.75, 0.85),
        2 => Color::srgb(0.35, 0.7, 0.95),
        _ => Color::srgb(0.9, 0.3, 0.3),
    }
}
