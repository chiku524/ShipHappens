//! Headless smoke — Nest boots and advances into a mode.

use std::{fs, path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    party::{HubReady, PartyDirector, PartyPhase},
    player::ThirdPersonCamera,
};

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
    pub written: bool,
    pub exit_requested: bool,
}

#[derive(Resource, Debug)]
struct SmokeAutomation {
    role: SmokeRole,
    timer: Timer,
    saw_client: bool,
    /// After a pass, host keeps the server up briefly so join can observe the phase change.
    exit_delay: Option<Timer>,
}

pub struct SmokeAutomationPlugin;

impl Plugin for SmokeAutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SmokeResult>()
            .add_systems(Startup, init_smoke_automation)
            .add_observer(on_remote_client_connected)
            .add_systems(
                Update,
                (
                    disable_cursor_for_smoke,
                    run_party_smoke,
                    finish_smoke,
                )
                    .chain(),
            );
    }
}

fn disable_cursor_for_smoke(mut camera: ResMut<ThirdPersonCamera>) {
    // Invisible / Xvfb windows cannot confine the cursor — avoid grab spam.
    // Keep forcing this: session_flow / pause code re-enables capture on player spawn.
    camera.captured = false;
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
                exit_delay: None,
            });
            return;
        }
    };

    commands.insert_resource(SmokeAutomation {
        role,
        timer: Timer::new(Duration::from_secs(45), TimerMode::Once),
        saw_client: false,
        exit_delay: None,
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

    // Host linger after pass so join can sync Race phase.
    if let Some(delay) = automation.exit_delay.as_mut() {
        delay.tick(time.delta());
        if delay.just_finished() {
            result.exit_requested = true;
        }
        return;
    }

    if result.finished {
        return;
    }

    automation.timer.tick(time.delta());

    // Host: only signal ready after a remote client connects (local sets saw_client at init).
    // Setting host_ready every frame used to auto-start FullParty before join arrived.
    if matches!(automation.role, SmokeRole::Host)
        && automation.saw_client
        && matches!(director.phase, PartyPhase::Hub)
    {
        ready.host_ready = true;
    }

    if !matches!(director.phase, PartyPhase::Hub) {
        result.pass = true;
        result.message = format!("party advanced to {:?}", director.phase);
        result.finished = true;
        if matches!(automation.role, SmokeRole::Host) {
            automation.exit_delay = Some(Timer::new(Duration::from_secs(20), TimerMode::Once));
            info!("smoke host pass — lingering 20s for join sync");
        } else {
            result.exit_requested = true;
        }
        return;
    }

    if automation.timer.just_finished() {
        result.pass = false;
        result.message = format!(
            "timed out waiting for party stage (role={:?}, saw_client={})",
            automation.role, automation.saw_client
        );
        result.finished = true;
        result.exit_requested = true;
    }
}

fn finish_smoke(
    mut result: ResMut<SmokeResult>,
    automation: Option<Res<SmokeAutomation>>,
    mut exit: MessageWriter<AppExit>,
) {
    if result.finished && !result.written {
        result.written = true;

        let role_name = automation
            .as_ref()
            .map(|a| match a.role {
                SmokeRole::Host => "host",
                SmokeRole::Join => "join",
            })
            .unwrap_or("unknown");

        let _ = fs::create_dir_all(SMOKE_RESULT_DIR);
        let path = PathBuf::from(SMOKE_RESULT_DIR).join(format!("mp_smoke_{role_name}.result"));
        let legacy = PathBuf::from(SMOKE_RESULT_DIR).join("smoke_result.txt");
        let body = format!(
            "pass={}\nmessage={}\nrole={role_name}\n",
            result.pass, result.message
        );
        let _ = fs::write(&path, &body);
        let _ = fs::write(&legacy, &body);
        info!(
            "smoke finished: {} — {} (wrote {})",
            result.pass,
            result.message,
            path.display()
        );
    }

    if result.exit_requested {
        result.exit_requested = false;
        if result.pass {
            exit.write(AppExit::Success);
        } else {
            exit.write(AppExit::from_code(1));
            // Ensure CI scripts see a non-zero status even if the winit runner drops AppExit.
            std::process::exit(1);
        }
    }
}
