use std::collections::HashMap;

use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    core::{
        CAMERA_DEFAULT_DISTANCE, CAMERA_MAX_DISTANCE, CAMERA_MIN_DISTANCE,
        MAX_CAMERA_PITCH, MIN_CAMERA_PITCH, MOUSE_SENSITIVITY,
        PLAYER_SPEED, PLAYER_SPRINT_MULTIPLIER,
    },
    network::OwnedPlayer,
    world::{GameplayEntity, MainCamera},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ThirdPersonCamera>()
            .add_observer(init_player_visuals)
            .add_systems(
                Update,
                (
                    assign_local_player,
                    capture_cursor_when_playing,
                    toggle_cursor_capture,
                    camera_mouse_look,
                    camera_scroll_zoom,
                    local_movement_input,
                    follow_camera.after(camera_mouse_look),
                ),
            );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ThirdPersonCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub captured: bool,
}

impl Default for ThirdPersonCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: -0.35,
            distance: CAMERA_DEFAULT_DISTANCE,
            captured: true,
        }
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkPlayer {
    pub slot: u32,
}

#[derive(Component, Serialize, Deserialize, Clone, Debug, Default)]
pub struct PlayerName(pub String);

#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct PlayerColor(pub [f32; 3]);

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct LocalPlayer;

#[derive(Resource, Default, Debug)]
pub struct PlayerRegistry {
    pub players: HashMap<Entity, Entity>,
    pub local_player: Option<Entity>,
}

#[derive(Event, Serialize, Deserialize, Clone, Copy, Debug)]
pub struct MoveInput {
    pub direction: Vec2,
    pub sprint: bool,
}

pub fn apply_move_input(
    input: On<FromClient<MoveInput>>,
    owners: Query<&OwnedPlayer>,
    mut players: Query<(&mut Transform, &NetworkPlayer)>,
    time: Res<Time>,
) {
    let Some(client_entity) = input.client_id.entity() else {
        return;
    };

    let Ok(owned) = owners.get(client_entity) else {
        return;
    };

    let Ok((mut transform, _)) = players.get_mut(owned.0) else {
        return;
    };

    let direction = Vec3::new(input.direction.x, 0.0, input.direction.y);
    if direction.length_squared() <= f32::EPSILON {
        return;
    }

    let speed = if input.sprint {
        PLAYER_SPEED * PLAYER_SPRINT_MULTIPLIER
    } else {
        PLAYER_SPEED
    };

    transform.translation += direction.normalize() * speed * time.delta_secs();
}

fn capture_cursor_when_playing(
    local_player: Query<(), With<LocalPlayer>>,
    camera: Res<ThirdPersonCamera>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if local_player.is_empty() || !camera.captured {
        return;
    }

    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };

    if cursor.grab_mode != CursorGrabMode::Locked {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn toggle_cursor_capture(
    keyboard: Res<ButtonInput<KeyCode>>,
    local_player: Query<(), With<LocalPlayer>>,
    mut camera: ResMut<ThirdPersonCamera>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if local_player.is_empty() || !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    camera.captured = !camera.captured;
    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };

    if camera.captured {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    } else {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn camera_mouse_look(
    local_player: Query<(), With<LocalPlayer>>,
    motion: Res<AccumulatedMouseMotion>,
    mut camera: ResMut<ThirdPersonCamera>,
) {
    if local_player.is_empty() || !camera.captured {
        return;
    }

    let delta = motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    camera.yaw -= delta.x * MOUSE_SENSITIVITY;
    camera.pitch = (camera.pitch - delta.y * MOUSE_SENSITIVITY)
        .clamp(MIN_CAMERA_PITCH, MAX_CAMERA_PITCH);
}

fn camera_scroll_zoom(
    local_player: Query<(), With<LocalPlayer>>,
    mut scroll: MessageReader<MouseWheel>,
    mut camera: ResMut<ThirdPersonCamera>,
) {
    if local_player.is_empty() {
        return;
    }

    for event in scroll.read() {
        camera.distance = (camera.distance - event.y * 0.5)
            .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
    }
}

fn camera_relative_direction(keyboard: &ButtonInput<KeyCode>, yaw: f32) -> Vec3 {
    let forward = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += right;
    }

    direction
}

fn local_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    cli: Res<crate::Cli>,
    camera: Res<ThirdPersonCamera>,
    local_player: Query<(), With<LocalPlayer>>,
) {
    if local_player.is_empty() {
        return;
    }

    let direction = camera_relative_direction(&keyboard, camera.yaw);
    if direction.length_squared() <= f32::EPSILON {
        return;
    }

    let sprint = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    if cli.is_online() {
        let flat = direction.normalize();
        commands.client_trigger(MoveInput {
            direction: Vec2::new(flat.x, flat.z),
            sprint,
        });
    }
}

pub fn offline_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    camera: Res<ThirdPersonCamera>,
    mut players: Query<&mut Transform, With<LocalPlayer>>,
    time: Res<Time>,
    cli: Res<crate::Cli>,
) {
    if cli.is_online() {
        return;
    }

    let Ok(mut transform) = players.single_mut() else {
        return;
    };

    let direction = camera_relative_direction(&keyboard, camera.yaw);
    if direction.length_squared() <= f32::EPSILON {
        return;
    }

    let sprint = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let speed = if sprint {
        PLAYER_SPEED * PLAYER_SPRINT_MULTIPLIER
    } else {
        PLAYER_SPEED
    };

    transform.translation += direction.normalize() * speed * time.delta_secs();
}

fn assign_local_player(
    mut commands: Commands,
    mut registry: ResMut<PlayerRegistry>,
    cli: Res<crate::Cli>,
    client_state: Option<Res<State<ClientState>>>,
    players: Query<(Entity, &NetworkPlayer), Without<LocalPlayer>>,
) {
    if !cli.is_online() || registry.local_player.is_some() {
        return;
    }

    let Some(client_state) = client_state else {
        return;
    };
    if *client_state.get() != ClientState::Connected {
        return;
    }

    if let Some((player, _)) = players.iter().max_by_key(|(_, owner)| owner.slot) {
        commands.entity(player).insert(LocalPlayer);
        registry.local_player = Some(player);
        info!("local player assigned: {player:?}");
    }
}

fn init_player_visuals(
    add: On<Add, NetworkPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    players: Query<&PlayerColor, With<NetworkPlayer>>,
) {
    let Ok(color) = players.get(add.entity) else {
        return;
    };

    let [r, g, b] = color.0;
    commands.entity(add.entity).insert((
        Mesh3d(meshes.add(Capsule3d::new(0.45, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            ..Default::default()
        })),
        GameplayEntity,
    ));
}

fn follow_camera(
    camera_state: Res<ThirdPersonCamera>,
    local_player: Query<&Transform, With<LocalPlayer>>,
    mut camera: Query<&mut Transform, (With<MainCamera>, Without<LocalPlayer>)>,
) {
    let Ok(player) = local_player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let yaw = camera_state.yaw;
    let pitch = camera_state.pitch;
    let distance = camera_state.distance;

    let horizontal = distance * pitch.cos();
    let eye = player.translation
        + Vec3::new(
            horizontal * yaw.sin(),
            -distance * pitch.sin() + 1.5,
            horizontal * yaw.cos(),
        );

    camera_transform.translation = eye;
    camera_transform.look_at(player.translation + Vec3::Y * 0.9, Vec3::Y);
}
