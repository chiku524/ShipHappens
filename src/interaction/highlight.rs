//! Floating ring that marks the nearest interactable station.

use bevy::prelude::*;

use crate::{
    player::LocalPlayer,
    rooms::LayoutMarkerId,
    world::GameplayEntity,
};

use super::{nearest_interactable, Interactable};

#[derive(Component)]
pub struct InteractHighlight;

pub fn spawn_interact_highlight(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        InteractHighlight,
        GameplayEntity,
        Mesh3d(meshes.add(Cylinder::new(0.85, 0.06))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.95, 0.85, 0.2, 0.55),
            emissive: LinearRgba::rgb(0.6, 0.45, 0.05),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -50.0, 0.0),
        Visibility::Hidden,
        Name::new("InteractHighlight"),
    ));
}

pub fn update_interact_highlight(
    time: Res<Time>,
    local_player: Query<&Transform, With<LocalPlayer>>,
    stations: Query<(Entity, &Transform, &Interactable, Option<&LayoutMarkerId>)>,
    mut highlight: Query<
        (&mut Transform, &mut Visibility),
        (With<InteractHighlight>, Without<LocalPlayer>, Without<Interactable>),
    >,
) {
    let Ok((mut highlight_tf, mut visibility)) = highlight.single_mut() else {
        return;
    };

    let Ok(player) = local_player.single() else {
        *visibility = Visibility::Hidden;
        return;
    };

    let Some((_, station_tf, _, _)) = nearest_interactable(player.translation, &stations) else {
        *visibility = Visibility::Hidden;
        return;
    };

    *visibility = Visibility::Visible;
    let pulse = 1.0 + (time.elapsed_secs() * 4.0).sin() * 0.08;
    highlight_tf.translation = Vec3::new(station_tf.translation.x, 0.08, station_tf.translation.z);
    highlight_tf.scale = Vec3::new(pulse, 1.0, pulse);
}
