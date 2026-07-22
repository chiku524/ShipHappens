//! Camera shake + flash + short spark bursts for interact feedback.

use bevy::prelude::*;

use crate::world::{GameplayEntity, MainCamera};

#[derive(Resource, Debug, Default)]
pub struct CameraShake {
    pub trauma: f32,
}

impl CameraShake {
    pub fn add(&mut self, amount: f32) {
        self.trauma = (self.trauma + amount).clamp(0.0, 1.0);
    }
}

#[derive(Resource, Debug, Default)]
pub struct FeedbackFlash {
    pub timer: f32,
    pub color: Color,
}

impl FeedbackFlash {
    pub fn trigger(&mut self, color: Color, secs: f32) {
        self.color = color;
        self.timer = secs;
    }
}

#[derive(Resource, Debug, Default)]
pub struct SparkQueue {
    pub bursts: Vec<(Vec3, Color)>,
}

impl SparkQueue {
    pub fn push(&mut self, at: Vec3, color: Color) {
        self.bursts.push((at, color));
    }
}

#[derive(Component)]
struct FlashOverlay;

#[derive(Component)]
struct SparkBurst {
    ttl: f32,
}

pub struct JuicePlugin;

impl Plugin for JuicePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraShake>()
            .init_resource::<FeedbackFlash>()
            .init_resource::<SparkQueue>()
            .add_systems(Startup, spawn_flash_overlay)
            .add_systems(
                PostUpdate,
                (tick_camera_shake, tick_feedback_flash, drain_spark_queue, tick_sparks),
            );
    }
}

fn spawn_flash_overlay(mut commands: Commands) {
    commands.spawn((
        FlashOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Default::default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(1000),
        Visibility::Hidden,
    ));
}

fn tick_camera_shake(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    let Ok(mut transform) = camera.single_mut() else {
        return;
    };

    if shake.trauma <= 0.001 {
        shake.trauma = 0.0;
        return;
    }

    let t = time.elapsed_secs() * 40.0;
    let intensity = shake.trauma * shake.trauma * 0.18;
    transform.translation.x += t.sin() * intensity;
    transform.translation.y += (t * 1.3).cos() * intensity * 0.6;
    shake.trauma = (shake.trauma - time.delta_secs() * 1.8).max(0.0);
}

fn tick_feedback_flash(
    time: Res<Time>,
    mut flash: ResMut<FeedbackFlash>,
    mut overlay: Query<(&mut BackgroundColor, &mut Visibility), With<FlashOverlay>>,
) {
    let Ok((mut bg, mut visibility)) = overlay.single_mut() else {
        return;
    };

    if flash.timer <= 0.0 {
        *visibility = Visibility::Hidden;
        bg.0 = Color::NONE;
        return;
    }

    flash.timer -= time.delta_secs();
    let alpha = (flash.timer * 2.5).clamp(0.0, 0.35);
    let mut c = flash.color;
    c.set_alpha(alpha);
    bg.0 = c;
    *visibility = Visibility::Visible;
}

fn drain_spark_queue(
    mut queue: ResMut<SparkQueue>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (at, color) in queue.bursts.drain(..) {
        for i in 0..6 {
            let angle = i as f32 * 1.05;
            let offset = Vec3::new(angle.cos() * 0.35, 0.4 + (i as f32) * 0.08, angle.sin() * 0.35);
            commands.spawn((
                SparkBurst { ttl: 0.35 },
                GameplayEntity,
                Mesh3d(meshes.add(Cuboid::new(0.12, 0.12, 0.12))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: LinearRgba::from(color) * 3.0,
                    unlit: true,
                    ..Default::default()
                })),
                Transform::from_translation(at + offset),
                Name::new("Spark"),
            ));
        }
    }
}

fn tick_sparks(
    time: Res<Time>,
    mut commands: Commands,
    mut sparks: Query<(Entity, &mut SparkBurst, &mut Transform)>,
) {
    for (entity, mut burst, mut tf) in &mut sparks {
        burst.ttl -= time.delta_secs();
        tf.translation.y += time.delta_secs() * 1.8;
        tf.scale *= 1.0 - time.delta_secs() * 2.2;
        if burst.ttl <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Kind of juice to play after an interact outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JuiceEvent {
    Pickup,
    SortOk,
    SortBad,
    GenericOk,
    GenericBad,
    RoomClear,
    MeltdownFail,
    Knockback,
}

pub fn play_juice(
    event: JuiceEvent,
    shake: &mut CameraShake,
    flash: &mut FeedbackFlash,
    audio: &mut crate::audio_fx::AudioFxQueue,
) {
    match event {
        JuiceEvent::Pickup => {
            flash.trigger(Color::srgba(0.95, 0.85, 0.3, 1.0), 0.15);
            audio.push(crate::audio_fx::FxKind::Pickup);
        }
        JuiceEvent::SortOk => {
            shake.add(0.25);
            flash.trigger(Color::srgba(0.3, 0.95, 0.45, 1.0), 0.2);
            audio.push(crate::audio_fx::FxKind::SortOk);
        }
        JuiceEvent::SortBad => {
            shake.add(0.55);
            flash.trigger(Color::srgba(0.95, 0.25, 0.2, 1.0), 0.28);
            audio.push(crate::audio_fx::FxKind::SortBad);
            audio.push(crate::audio_fx::FxKind::Knockback);
        }
        JuiceEvent::GenericOk => {
            shake.add(0.15);
            flash.trigger(Color::srgba(0.4, 0.8, 1.0, 1.0), 0.12);
            audio.push(crate::audio_fx::FxKind::Ok);
        }
        JuiceEvent::GenericBad => {
            shake.add(0.4);
            flash.trigger(Color::srgba(0.9, 0.4, 0.1, 1.0), 0.2);
            audio.push(crate::audio_fx::FxKind::Bad);
            audio.push(crate::audio_fx::FxKind::Knockback);
        }
        JuiceEvent::RoomClear => {
            shake.add(0.35);
            flash.trigger(Color::srgba(0.35, 1.0, 0.55, 1.0), 0.55);
            audio.push(crate::audio_fx::FxKind::RoomClear);
        }
        JuiceEvent::MeltdownFail => {
            shake.add(0.9);
            flash.trigger(Color::srgba(1.0, 0.15, 0.05, 1.0), 0.8);
            audio.push(crate::audio_fx::FxKind::MeltdownFail);
        }
        JuiceEvent::Knockback => {
            shake.add(0.45);
            audio.push(crate::audio_fx::FxKind::Knockback);
        }
    }
}

pub fn juice_applies_knockback(event: JuiceEvent) -> bool {
    matches!(
        event,
        JuiceEvent::SortBad | JuiceEvent::GenericBad | JuiceEvent::Knockback
    )
}

pub fn juice_spark_color(event: JuiceEvent) -> Option<Color> {
    match event {
        JuiceEvent::SortBad | JuiceEvent::GenericBad | JuiceEvent::Knockback => {
            Some(Color::srgb(1.0, 0.55, 0.15))
        }
        JuiceEvent::SortOk | JuiceEvent::RoomClear => Some(Color::srgb(0.35, 1.0, 0.55)),
        JuiceEvent::MeltdownFail => Some(Color::srgb(1.0, 0.2, 0.05)),
        _ => None,
    }
}
