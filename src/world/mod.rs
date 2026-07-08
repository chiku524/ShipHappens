use bevy::prelude::*;

use crate::{
    assets::spawn_job_station,
    core::{
        BREAKER_PANEL_ASSET, CRANE_CONSOLE_ASSET, POWER_HOUR_SEQUENCE,
    },
    data::StudioRegistry,
    interaction::Interactable,
};

/// Marker for entities spawned as part of the greybox level.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameplayEntity;

/// Marks the main viewport camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        MainCamera,
        Transform::from_xyz(0.0, 8.0, 14.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("MainCamera"),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..Default::default()
        },
        Transform::from_xyz(4.0, 12.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_greybox_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    registry: Res<StudioRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.18, 0.20, 0.24),
        ..Default::default()
    });
    let floor_mesh = meshes.add(Cuboid::new(40.0, 0.5, 40.0));

    commands.spawn((
        GameplayEntity,
        Mesh3d(floor_mesh),
        MeshMaterial3d(floor_material),
        Transform::from_xyz(0.0, -0.25, 0.0),
        Name::new("Floor"),
    ));

    spawn_job_station(
        &mut commands,
        &asset_server,
        registry.as_ref(),
        &mut meshes,
        &mut materials,
        CRANE_CONSOLE_ASSET,
        Transform::from_xyz(0.0, 0.0, -6.0),
        Interactable::crane(),
        Color::srgb(0.85, 0.55, 0.15),
        Vec3::new(1.5, 1.2, 1.0),
    );

    let breaker_positions = [
        Vec3::new(10.0, 0.0, -4.0),
        Vec3::new(10.0, 0.0, -1.0),
        Vec3::new(10.0, 0.0, 2.0),
        Vec3::new(10.0, 0.0, 5.0),
    ];

    for (index, position) in breaker_positions.into_iter().enumerate() {
        spawn_job_station(
            &mut commands,
            &asset_server,
            registry.as_ref(),
            &mut meshes,
            &mut materials,
            BREAKER_PANEL_ASSET,
            Transform::from_xyz(position.x, position.y, position.z)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            Interactable::breaker(index as u8),
            Color::srgb(0.35, 0.55, 0.95),
            Vec3::new(0.8, 1.2, 0.4),
        );
    }

    info!(
        "spawned crane + {} power hour breakers (sequence {:?})",
        POWER_HOUR_SEQUENCE.len(),
        POWER_HOUR_SEQUENCE
    );
}
