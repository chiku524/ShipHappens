//! Headless smoke — Nest boots and advances into a mode.

use std::{fs, path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::party::{HubReady, PartyDirector, PartyPhase};

pub const SMOKE_RESULT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/.bevy");

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmokeRole {
    Host,
    Join,
}

#[derive(Resource, Default, Debug)]
pub struct SmokeResult {
    pub pass: bool,
    pub message: String,
    pub finished: bool,
}

#[derive(Resource, Debug)]
struct SmokeAutomation {
    role: SmokeRole,
    timer: Timer,
    saw_client: bool,
}

pub struct SmokeAutomationPlugin;

impl Plugin for SmokeAutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SmokeResult>()
            .add_systems(Startup, init_smoke_automation)
            .add_observer(on_remote_client_connected)
            .add_systems(Update, (run_party_smoke, finish_smoke));
    }
}

fn on_remote_client_connected(
    _add: On<Add, ConnectedClient>,
    automation: Option<ResMut<SmokeAutomation>>,
) {
    let Some(mut automation) = automation else {
        return;
    };
    if automation.role == SmokeRole::Host {
        automation.saw_client = true;
        info!("smoke: remote client connected");
    }
}

fn init_smoke_automation(mut commands: Commands, cli: Res<crate::Cli>) {
    let role = match *cli {
        crate::Cli::Host { .. } => SmokeRole::Host,
        crate::Cli::Join { .. } => SmokeRole::Join,
        crate::Cli::Local => {
            commands.insert_resource(SmokeAutomation {
                role: SmokeRole::Host,
                timer: Timer::new(Duration::from_secs(25), TimerMode::Once),
                saw_client: true,
            });
            return;
        }
    };

    commands.insert_resource(SmokeAutomation {
        role,
        timer: Timer::new(Duration::from_secs(30), TimerMode::Once),
        saw_client: false,
    });
}

fn run_party_smoke(
    time: Res<Time>,
    automation: Option<ResMut<SmokeAutomation>>,
    mut ready: ResMut<HubReady>,
    director: Res<PartyDirector>,
    mut result: ResMut<SmokeResult>,
) {
    let Some(mut automation) = automation else {
        return;
    };
    if result.finished {
        return;
    }

    automation.timer.tick(time.delta());
    if matches!(automation.role, SmokeRole::Host) {
        ready.host_ready = true;
    }

    if !matches!(director.phase, PartyPhase::Hub) {
        result.pass = true;
        result.message = format!("party advanced to {:?}", director.phase);
        result.finished = true;
        return;
    }

    if automation.timer.just_finished() {
        result.pass = false;
        result.message = "timed out waiting for party stage".into();
        result.finished = true;
    }
}

fn finish_smoke(result: Res<SmokeResult>, mut exit: MessageWriter<AppExit>) {
    if !result.finished || !result.is_changed() {
        return;
    }
    let _ = fs::create_dir_all(SMOKE_RESULT_DIR);
    let path = PathBuf::from(SMOKE_RESULT_DIR).join("smoke_result.txt");
    let body = format!(
        "pass={}\nmessage={}\n",
        result.pass, result.message
    );
    let _ = fs::write(&path, body);
    info!("smoke finished: {} — {}", result.pass, result.message);
    if result.pass {
        exit.write(AppExit::Success);
    } else {
        exit.write(AppExit::from_code(1));
    }
}
