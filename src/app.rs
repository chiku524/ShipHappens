use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::RepliconRenetPlugins;

use crate::{
    announcer::AnnouncerPlugin,
    assets::{load_studio_registry, AssetsPlugin},
    cli::Cli,
    data::load_job_manifest,
    economy::EconomyPlugin,
    interaction::InteractionPlugin,
    jobs::JobSystem,
    live_ops::LiveOpsPlugin,
    meta::MetaPlugin,
    network::{init_network_backend, setup_job_board, spawn_offline_player, NetworkPlugin},
    player::{offline_movement, PlayerPlugin},
    rooms::{assign_leaseholder, RoomsPlugin},
    scoring::ScoringPlugin,
    smoke::SmokeAutomationPlugin,
    tournament::TournamentPlugin,
    ui::UiPlugin,
    world::{spawn_camera, spawn_greybox_level, WorldPlugin},
};

/// Shared app builder for interactive and headless smoke binaries.
pub fn build_app(headless: bool, enable_smoke: bool) -> App {
    let asset_root = format!("{}/assets", env!("CARGO_MANIFEST_DIR"));
    let mut app = App::new();
    app.init_resource::<Cli>();

    let window_plugin = if headless {
        WindowPlugin {
            primary_window: Some(Window {
                title: "ShipHappens Smoke".into(),
                resolution: (1u32, 1u32).into(),
                visible: false,
                ..default()
            }),
            ..default()
        }
    } else {
        WindowPlugin {
            primary_window: Some(Window {
                title: "ShipHappens — Vault Break".into(),
                ..default()
            }),
            ..default()
        }
    };

    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: asset_root,
            ..default()
        })
        .set(window_plugin);

    app.add_plugins((default_plugins, RepliconPlugins, RepliconRenetPlugins));

    app.add_plugins((
        AssetsPlugin,
        WorldPlugin,
        NetworkPlugin,
        PlayerPlugin,
        InteractionPlugin,
        ScoringPlugin,
        RoomsPlugin,
        TournamentPlugin,
        EconomyPlugin,
        AnnouncerPlugin,
        MetaPlugin,
        LiveOpsPlugin,
    ));

    if !headless {
        app.add_plugins(UiPlugin);
    }

    app.add_systems(
        Startup,
        (
            load_studio_registry,
            load_manifest,
            setup_job_board,
            spawn_camera,
            spawn_greybox_level,
            init_network_backend,
            spawn_offline_player,
        )
            .chain(),
    )
    .add_systems(Update, (offline_movement, assign_leaseholder));

    if enable_smoke {
        app.add_plugins(SmokeAutomationPlugin);
    }

    app
}

fn load_manifest(mut commands: Commands) {
    let path = format!("{}/data/job_manifest.json", env!("CARGO_MANIFEST_DIR"));
    match load_job_manifest(&path) {
        Ok(definitions) => {
            info!("loaded {} jobs from manifest", definitions.len());
            commands.insert_resource(JobSystem::from_definitions(definitions));
        }
        Err(err) => {
            panic!("failed to load job manifest: {err}");
        }
    }
}
