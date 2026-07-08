use bevy::prelude::*;

/// Persistent arena shell — walls/floor/ceiling slots from `data/rooms/arena.json`.
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

pub fn spawn_arena_shell(
    mut commands: Commands,
    arena: Res<crate::data::ArenaLayout>,
    asset_server: Res<AssetServer>,
    registry: Res<crate::data::StudioRegistry>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for marker in &arena.0.markers {
        crate::rooms::spawner::spawn_arena_marker(
            &mut commands,
            &asset_server,
            registry.as_ref(),
            &mut meshes,
            &mut materials,
            marker,
        );
    }

    info!(
        "spawned arena shell: {} ({} markers)",
        arena.0.label,
        arena.0.markers.len()
    );
}

/// Marks the main viewport camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct MainCamera;

/// Marker for entities spawned as part of the greybox level.
#[derive(Component, Debug, Clone, Copy)]
pub struct GameplayEntity;

/// Persistent arena geometry — not despawned when vault stages swap.
#[derive(Component, Debug, Clone, Copy)]
pub struct ArenaPiece;
