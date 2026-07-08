use bevy::prelude::*;

use crate::data::{MarkerMotionSpec, MotionPresetKind};

use super::StationKind;

/// Rest pose captured at spawn — motion tweens are applied relative to this.
#[derive(Component, Debug, Clone)]
pub struct InteractMotion {
    pub rest: Transform,
    pub spec: Option<MarkerMotionSpec>,
}

/// Future hook: when set, a skeletal clip should play on the GLB scene root.
#[derive(Component, Debug, Clone)]
pub struct GlbInteractClip(pub String);

#[derive(Component, Debug, Clone)]
pub struct ActiveInteractMotion {
    pub preset: MotionPreset,
    pub elapsed: f32,
    pub duration: f32,
    pub success: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum MotionPreset {
    BreakerFlip,
    BreakerZap,
    ValveTurn,
    DoorSeal,
    SortChutePulse,
    VaultBob,
    CraneNudge,
    ZonePulse,
}

impl From<MotionPresetKind> for MotionPreset {
    fn from(kind: MotionPresetKind) -> Self {
        match kind {
            MotionPresetKind::BreakerFlip => Self::BreakerFlip,
            MotionPresetKind::BreakerZap => Self::BreakerZap,
            MotionPresetKind::ValveTurn => Self::ValveTurn,
            MotionPresetKind::DoorSeal => Self::DoorSeal,
            MotionPresetKind::SortChutePulse => Self::SortChutePulse,
            MotionPresetKind::VaultBob => Self::VaultBob,
            MotionPresetKind::CraneNudge => Self::CraneNudge,
            MotionPresetKind::ZonePulse => Self::ZonePulse,
            MotionPresetKind::None => Self::VaultBob, // unused — resolve returns None first
        }
    }
}

impl MotionPreset {
    pub fn from_station(kind: StationKind, success: bool) -> Self {
        match kind {
            StationKind::PowerHourBreaker { .. } => {
                if success {
                    Self::BreakerFlip
                } else {
                    Self::BreakerZap
                }
            }
            StationKind::CoolantValve { .. } => Self::ValveTurn,
            StationKind::MeltdownDoor { .. } => Self::DoorSeal,
            StationKind::SortChute { .. } => Self::SortChutePulse,
            StationKind::VaultObjective => Self::VaultBob,
            StationKind::CraneConsole => Self::CraneNudge,
        }
    }

    pub fn duration(self) -> f32 {
        match self {
            Self::BreakerFlip => 0.35,
            Self::BreakerZap => 0.45,
            Self::ValveTurn => 0.5,
            Self::DoorSeal => 0.55,
            Self::SortChutePulse => 0.3,
            Self::VaultBob => 0.4,
            Self::CraneNudge => 0.45,
            Self::ZonePulse => 0.35,
        }
    }

    fn sample(&self, rest: &Transform, t: f32, success: bool) -> Transform {
        let u = (t / self.duration()).clamp(0.0, 1.0);
        let ease = smoothstep(u);

        match self {
            Self::BreakerFlip => {
                let angle = -50.0_f32.to_radians() * ease;
                let mut out = *rest;
                out.rotation = rest.rotation * Quat::from_rotation_x(angle);
                out
            }
            Self::BreakerZap => {
                let shake = (u * std::f32::consts::TAU * 6.0).sin() * (1.0 - u) * 0.15;
                let mut out = *rest;
                out.translation += Vec3::new(shake, shake.abs() * 0.05, -shake * 0.5);
                out
            }
            Self::ValveTurn => {
                let angle = 90.0_f32.to_radians() * ease;
                let mut out = *rest;
                out.rotation = rest.rotation * Quat::from_rotation_y(angle);
                out
            }
            Self::DoorSeal => {
                let slide = ease * 1.2;
                let yaw = 35.0_f32.to_radians() * ease;
                let mut out = *rest;
                out.translation += rest.rotation * Vec3::new(slide, 0.0, 0.0);
                out.rotation = rest.rotation * Quat::from_rotation_y(yaw);
                out
            }
            Self::SortChutePulse => {
                let scale = 1.0 + 0.12 * (std::f32::consts::PI * ease).sin();
                let bob = 0.15 * (std::f32::consts::PI * ease).sin();
                let mut out = *rest;
                out.translation.y += bob;
                out.scale = rest.scale * scale;
                if !success {
                    out.translation.x += (u * 8.0).sin() * 0.05 * (1.0 - u);
                }
                out
            }
            Self::VaultBob => {
                let bob = 0.35 * (std::f32::consts::PI * ease).sin();
                let mut out = *rest;
                out.translation.y += bob;
                out
            }
            Self::CraneNudge => {
                let yaw = 8.0_f32.to_radians() * (std::f32::consts::PI * ease).sin();
                let mut out = *rest;
                out.rotation = rest.rotation * Quat::from_rotation_y(yaw);
                out
            }
            Self::ZonePulse => {
                let scale = 1.0 + 0.08 * (std::f32::consts::PI * ease).sin();
                let mut out = *rest;
                out.scale = Vec3::new(rest.scale.x * scale, rest.scale.y, rest.scale.z * scale);
                out
            }
        }
    }
}

pub struct ResolvedMotion {
    pub preset: MotionPreset,
    pub duration: f32,
    pub glb_clip: Option<String>,
}

pub fn resolve_motion(
    spec: Option<&MarkerMotionSpec>,
    station_kind: StationKind,
    success: bool,
) -> Option<ResolvedMotion> {
    if let Some(spec) = spec {
        if matches!(spec.preset, Some(MotionPresetKind::None)) {
            return None;
        }

        let preset_kind = if success {
            spec.preset
        } else {
            spec.fail_preset.or(spec.preset)
        };

        let preset = match preset_kind {
            Some(preset) if preset != MotionPresetKind::None => MotionPreset::from(preset),
            _ => MotionPreset::from_station(station_kind, success),
        };

        let duration = spec.duration_secs.unwrap_or_else(|| preset.duration());
        let glb_clip = if success {
            spec.glb_clip.clone()
        } else {
            None
        };

        return Some(ResolvedMotion {
            preset,
            duration,
            glb_clip,
        });
    }

    let preset = MotionPreset::from_station(station_kind, success);
    Some(ResolvedMotion {
        preset,
        duration: preset.duration(),
        glb_clip: None,
    })
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn attach_interact_motion(
    commands: &mut Commands,
    entity: Entity,
    rest: Transform,
    spec: Option<MarkerMotionSpec>,
) {
    commands.entity(entity).insert(InteractMotion { rest, spec });
}

/// Start or restart a procedural interact animation on the station entity.
pub fn trigger_interact_motion(
    commands: &mut Commands,
    entity: Entity,
    kind: StationKind,
    success: bool,
    motion: &Query<&InteractMotion>,
) {
    let Ok(anchor) = motion.get(entity) else {
        return;
    };

    let Some(resolved) = resolve_motion(anchor.spec.as_ref(), kind, success) else {
        return;
    };

    if let Some(clip) = resolved.glb_clip {
        commands.entity(entity).insert(GlbInteractClip(clip));
    }

    commands.entity(entity).insert(ActiveInteractMotion {
        preset: resolved.preset,
        elapsed: 0.0,
        duration: resolved.duration,
        success,
    });
    commands.entity(entity).insert(anchor.rest);
}

pub fn tick_interact_motion(
    time: Res<Time>,
    mut commands: Commands,
    mut movers: Query<(Entity, &InteractMotion, &mut Transform, &mut ActiveInteractMotion)>,
) {
    let dt = time.delta_secs();
    for (entity, anchor, mut transform, mut active) in &mut movers {
        active.elapsed += dt;
        let t = active.elapsed.min(active.duration);
        *transform = active.preset.sample(&anchor.rest, t, active.success);

        if active.elapsed >= active.duration {
            *transform = anchor.rest;
            commands
                .entity(entity)
                .remove::<ActiveInteractMotion>()
                .remove::<GlbInteractClip>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::MarkerMotionSpec;

    #[test]
    fn breaker_flip_eases_to_angle() {
        let rest = Transform::default();
        let end = MotionPreset::BreakerFlip.sample(&rest, 0.35, true);
        assert!(end.rotation != rest.rotation);
    }

    #[test]
    fn marker_motion_overrides_duration() {
        let spec = MarkerMotionSpec {
            preset: Some(MotionPresetKind::ValveTurn),
            fail_preset: None,
            duration_secs: Some(0.9),
            glb_clip: Some("turn_wheel".into()),
        };
        let resolved = resolve_motion(Some(&spec), StationKind::CoolantValve { index: 0 }, true)
            .expect("motion resolves");
        assert_eq!(resolved.duration, 0.9);
        assert_eq!(resolved.glb_clip.as_deref(), Some("turn_wheel"));
    }

    #[test]
    fn none_preset_disables_motion() {
        let spec = MarkerMotionSpec {
            preset: Some(MotionPresetKind::None),
            fail_preset: None,
            duration_secs: None,
            glb_clip: None,
        };
        assert!(resolve_motion(Some(&spec), StationKind::VaultObjective, true).is_none());
    }
}
