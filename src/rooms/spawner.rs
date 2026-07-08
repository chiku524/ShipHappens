use bevy::prelude::*;

use crate::{
    assets::{spawn_decoration, spawn_job_station},
    data::{GreyboxSpec, InteractableSpec, LayoutMarker, StudioRegistry},
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
        if glb_exists(registry, asset_id) {
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
    match tag {
        MarkerTag::Room => {
            commands.entity(entity).insert((
                RoomLayoutPiece,
                LayoutMarkerId(marker_id.to_string()),
            ));
        }
        MarkerTag::Arena => {
            commands
                .entity(entity)
                .insert((ArenaPiece, Name::new(format!("Arena_{marker_id}"))));
        }
    }
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
    let mat = greybox_material(materials, greybox);
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

pub fn marker_transform(marker: &LayoutMarker) -> Transform {
    Transform::from_xyz(marker.position[0], marker.position[1], marker.position[2])
        .with_rotation(Quat::from_rotation_y(marker.rotation_y_deg.to_radians()))
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
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: Color::srgb(greybox.color[0], greybox.color[1], greybox.color[2]),
        emissive: greybox.emissive.map_or(LinearRgba::BLACK, |e| {
            LinearRgba::rgb(e[0], e[1], e[2])
        }),
        alpha_mode: if greybox.emissive.is_some() && greybox.size[1] < 0.2 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
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

fn glb_exists(registry: &StudioRegistry, asset_id: &str) -> bool {
    let glb_path = registry.glb_asset_path(asset_id);
    let full_path = format!("{}/assets/{glb_path}", env!("CARGO_MANIFEST_DIR"));
    std::path::Path::new(&full_path).exists()
}
