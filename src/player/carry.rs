//! Held freight for HR Orientation sort loop.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::rooms::RoomRuntime;
use crate::world::GameplayEntity;

/// Freight currently held by a player. `kind` matches sort chute index 0–3.
#[derive(Component, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarryingFreight {
    pub kind: u8,
}

impl CarryingFreight {
    pub fn label(self) -> &'static str {
        RoomRuntime::sort_label(self.kind)
    }
}

#[derive(Component)]
pub struct CarriedFreightVisual;

/// Sync a small crate mesh on the local/authority player while carrying.
pub fn sync_carry_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    carriers: Query<
        (Entity, Option<&CarryingFreight>, Option<&Children>),
        (
            With<crate::player::NetworkPlayer>,
            Or<(
                With<crate::player::LocalPlayer>,
                Changed<CarryingFreight>,
            )>,
        ),
    >,
    visuals: Query<Entity, With<CarriedFreightVisual>>,
) {
    for (entity, carrying, children) in &carriers {
        let existing = children
            .map(|c| {
                c.iter()
                    .find(|child| visuals.get(*child).is_ok())
            })
            .flatten();

        match (carrying, existing) {
            (None, Some(visual)) => {
                commands.entity(visual).despawn();
            }
            (Some(freight), None) => {
                let color = freight_color(freight.kind);
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        CarriedFreightVisual,
                        GameplayEntity,
                        Mesh3d(meshes.add(Cuboid::new(0.45, 0.45, 0.45))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: color,
                            ..Default::default()
                        })),
                        Transform::from_xyz(0.35, 0.85, 0.35),
                        Name::new(format!("Carry_{}", freight.label())),
                    ));
                });
            }
            (Some(freight), Some(visual)) => {
                // Update color if kind changed.
                commands.entity(visual).insert(MeshMaterial3d(
                    materials.add(StandardMaterial {
                        base_color: freight_color(freight.kind),
                        ..Default::default()
                    }),
                ));
            }
            (None, None) => {}
        }
    }
}

fn freight_color(kind: u8) -> Color {
    match kind {
        0 => Color::srgb(0.95, 0.45, 0.15),
        1 => Color::srgb(0.75, 0.75, 0.85),
        2 => Color::srgb(0.35, 0.7, 0.95),
        _ => Color::srgb(0.9, 0.3, 0.3),
    }
}
