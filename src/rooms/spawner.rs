use bevy::prelude::*;

use crate::{
    assets::{spawn_decoration, spawn_job_station, studio_asset_exists},
    data::{GreyboxSpec, InteractableSpec, LayoutMarker, MarkerRole, StudioRegistry},
    interaction::{attach_interact_motion, Interactable},
    world::{ArenaPiece, GameplayEntity},
};

use super::{LayoutMarkerId, RoomLayoutPiece};

pub fn spawn_layout_marker(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &StudioRegistry,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    marker: &LayoutMarker,
    tag: MarkerTag,
) {
    let transform = marker_transform(marker);
    let interactable = marker.interactable.as_ref().map(interactable_from_spec);
    let greybox = marker.greybox.as_ref();
    let name = greybox
        .and_then(|g| g.label.clone())
        .unwrap_or_else(|| marker.id.clone());

    let entity = if let Some(asset_id) = marker.asset_id.as_deref() {
        if studio_asset_exists(registry, asset_id) {
            if let Some(interactable) = interactable {
                spawn_job_station(
                    commands,
                    asset_server,
                    registry,
                    meshes,
                    materials,
                    asset_id,
                    transform,
                    interactable,
                    greybox_color(greybox),
                    greybox_size(greybox),
                )
            } else {
                spawn_decoration(commands, asset_server, registry, asset_id, transform)
                    .unwrap_or_else(|| {
                        spawn_greybox_only(
                            commands,
                            meshes,
                            materials,
                            marker,
                            transform,
                            None,
                            &name,
                        )
                    })
            }
        } else if greybox.is_some() {
            info!(
                "marker `{}` asset `{}` missing — greybox fallback",
                marker.id, asset_id
            );
            spawn_greybox_only(
                commands,
                meshes,
                materials,
                marker,
                transform,
                interactable,
                &name,
            )
        } else {
            warn!(
                "marker `{}` asset `{}` missing and no greybox",
                marker.id, asset_id
            );
            return;
        }
    } else if greybox.is_some() {
        spawn_greybox_only(
            commands,
            meshes,
            materials,
            marker,
            transform,
            interactable,
            &name,
        )
    } else {
        warn!(
            "layout marker `{}` has no asset_id and no greybox — skipped",
            marker.id
        );
        return;
    };

    tag_marker(commands, entity, &marker.id, tag);
    maybe_attach_push_volume(commands, entity, marker);

    if interactable.is_some() {
        attach_interact_motion(commands, entity, transform, marker.motion.clone());
    }
}

pub fn spawn_arena_marker(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &StudioRegistry,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    marker: &LayoutMarker,
) {
    spawn_layout_marker(
        commands,
        asset_server,
        registry,
        meshes,
        materials,
        marker,
        MarkerTag::Arena,
    );
}

#[derive(Debug, Clone, Copy)]
pub enum MarkerTag {
    Room,
    Arena,
}

fn tag_marker(commands: &mut Commands, entity: Entity, marker_id: &str, tag: MarkerTag) {
    // Always stamp LayoutMarkerId so interact RPCs can resolve by stable id.
    commands
        .entity(entity)
        .insert(LayoutMarkerId(marker_id.to_string()));

    match tag {
        MarkerTag::Room => {
            commands.entity(entity).insert(RoomLayoutPiece);
        }
        MarkerTag::Arena => {
            commands
                .entity(entity)
                .insert((ArenaPiece, Name::new(format!("Arena_{marker_id}"))));
        }
    }
}

fn maybe_attach_push_volume(
    commands: &mut Commands,
    entity: Entity,
    marker: &LayoutMarker,
) {
    let Some(greybox) = marker.greybox.as_ref() else {
        return;
    };
    // Floors / zones / VFX are walkable; walls / props / stations push.
    if matches!(
        marker.role,
        MarkerRole::Floor | MarkerRole::FloorDetail | MarkerRole::FloorVfx | MarkerRole::Zone
    ) {
        return;
    }
    let size = Vec3::new(greybox.size[0], greybox.size[1], greybox.size[2]) * marker.scale.max(0.01);
    if size.y < 0.35 {
        return;
    }
    commands.entity(entity).insert(crate::player::collision::PushVolume::from_greybox_size(size));
}

fn spawn_greybox_only(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    marker: &LayoutMarker,
    transform: Transform,
    interactable: Option<Interactable>,
    name: &str,
) -> Entity {
    let greybox = marker.greybox.as_ref().expect("greybox required");
    let mat = greybox_material(materials, greybox, marker.role);
    let mesh = meshes.add(Cuboid::new(
        greybox.size[0],
        greybox.size[1],
        greybox.size[2],
    ));

    let mut entity = commands.spawn((
        GameplayEntity,
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        transform,
        Name::new(name.to_string()),
    ));

    if let Some(interactable) = interactable {
        entity.insert(interactable);
    }

    entity.id()
}

/// World transform for a layout marker.
///
/// Greybox-only markers are lifted by half height (Bevy cuboids are center-origin).
/// GLB markers keep the authored Y — Tripo meshes are expected to be floor-pivoted.
pub fn marker_transform(marker: &LayoutMarker) -> Transform {
    let has_glb = marker.asset_id.as_ref().is_some_and(|id| !id.is_empty());
    let y_lift = if has_glb {
        0.0
    } else {
        marker
            .greybox
            .as_ref()
            .map(|g| g.size[1] * 0.5)
            .unwrap_or(0.0)
    };
    let scale = marker.scale.max(0.01);
    Transform::from_xyz(
        marker.position[0],
        marker.position[1] + y_lift,
        marker.position[2],
    )
    .with_rotation(Quat::from_rotation_y(marker.rotation_y_deg.to_radians()))
    .with_scale(Vec3::splat(scale))
}

pub fn interactable_from_spec(spec: &InteractableSpec) -> Interactable {
    match spec {
        InteractableSpec::Crane => Interactable::crane(),
        InteractableSpec::VaultObjective => Interactable::vault_objective(),
        InteractableSpec::SortChute { index } => Interactable::sort_chute(*index),
        InteractableSpec::Breaker { index } => Interactable::breaker(*index),
        InteractableSpec::CoolantValve { index } => Interactable::coolant_valve(*index),
        InteractableSpec::MeltdownDoor { index } => Interactable::meltdown_door(*index),
    }
}

fn greybox_material(
    materials: &mut Assets<StandardMaterial>,
    greybox: &GreyboxSpec,
    role: MarkerRole,
) -> Handle<StandardMaterial> {
    let thin_emissive = greybox.emissive.is_some() && greybox.size[1] < 0.2;
    let alpha = match role {
        MarkerRole::FloorVfx | MarkerRole::Zone if thin_emissive || greybox.emissive.is_some() => {
            AlphaMode::Blend
        }
        _ if thin_emissive => AlphaMode::Blend,
        _ => AlphaMode::Opaque,
    };

    materials.add(StandardMaterial {
        base_color: Color::srgb(greybox.color[0], greybox.color[1], greybox.color[2]),
        emissive: greybox.emissive.map_or(LinearRgba::BLACK, |e| {
            LinearRgba::rgb(e[0], e[1], e[2])
        }),
        alpha_mode: alpha,
        unlit: matches!(role, MarkerRole::FloorVfx | MarkerRole::Sign),
        ..Default::default()
    })
}

fn greybox_color(greybox: Option<&GreyboxSpec>) -> Color {
    greybox.map_or(Color::srgb(0.5, 0.5, 0.5), |g| {
        Color::srgb(g.color[0], g.color[1], g.color[2])
    })
}

fn greybox_size(greybox: Option<&GreyboxSpec>) -> Vec3 {
    greybox.map_or(Vec3::ONE, |g| Vec3::new(g.size[0], g.size[1], g.size[2]))
}
