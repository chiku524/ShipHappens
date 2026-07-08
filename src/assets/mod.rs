use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;

use crate::{
    data::StudioRegistry,
    interaction::Interactable,
    world::GameplayEntity,
};

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, _app: &mut App) {}
}

pub fn load_studio_registry(mut commands: Commands) {
    let path = format!("{}/assets/studio_registry.json", env!("CARGO_MANIFEST_DIR"));
    match StudioRegistry::load(&path) {
        Ok(registry) => {
            info!(
                "loaded {} Immersive Studio assets from registry",
                registry.assets.len()
            );
            commands.insert_resource(registry);
        }
        Err(err) => {
            warn!("studio registry unavailable ({err}); using fallback scales");
            commands.insert_resource(StudioRegistry {
                import_root: "models".into(),
                assets: Vec::new(),
            });
        }
    }
}

/// Spawns a GLB scene scaled to registry `target_height`, or a greybox cuboid on failure.
pub fn spawn_studio_prop(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &StudioRegistry,
    asset_id: &str,
    transform: Transform,
    bundle: impl Bundle,
) -> Entity {
    let target_height = registry.target_height(asset_id).unwrap_or(1.0);
    let scale = target_height.max(0.1);
    let glb_path = registry.glb_asset_path(asset_id);
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(glb_path.clone()));

    commands
        .spawn((
            GameplayEntity,
            WorldAssetRoot(scene),
            transform.with_scale(Vec3::splat(scale)),
            Name::new(asset_id.to_string()),
            bundle,
        ))
        .id()
}

/// Greybox fallback when GLB is unavailable (headless CI without assets).
pub fn spawn_greybox_prop(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    color: Color,
    size: Vec3,
    transform: Transform,
    bundle: impl Bundle,
) -> Entity {
    let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
    let material = materials.add(StandardMaterial {
        base_color: color,
        ..Default::default()
    });

    commands
        .spawn((
            GameplayEntity,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            bundle,
        ))
        .id()
}

/// Prefer GLB; attach interactable metadata either way.
pub fn spawn_job_station(
    commands: &mut Commands,
    asset_server: &AssetServer,
    registry: &StudioRegistry,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_id: &str,
    transform: Transform,
    interactable: Interactable,
    greybox_color: Color,
    greybox_size: Vec3,
) -> Entity {
    let glb_path = registry.glb_asset_path(asset_id);
    let full_path = format!("{}/assets/{glb_path}", env!("CARGO_MANIFEST_DIR"));
    if std::path::Path::new(&full_path).exists() {
        return spawn_studio_prop(
            commands,
            asset_server,
            registry,
            asset_id,
            transform,
            interactable,
        );
    }

    warn!("GLB missing at {full_path}, using greybox");
    spawn_greybox_prop(
        commands,
        meshes,
        materials,
        greybox_color,
        greybox_size,
        transform,
        interactable,
    )
}
