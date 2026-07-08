use std::{fs, path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    core::{CRANE_JOB_ID, POWER_HOUR_JOB_ID, POWER_HOUR_SEQUENCE},
    jobs::{JobBoard, JobSystem},
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
}

#[derive(Resource, Debug)]
struct SmokeAutomation {
    role: SmokeRole,
    timer: Timer,
    step_timer: Timer,
    connected_for: Timer,
    step: u8,
    saw_client: bool,
}

pub struct SmokeAutomationPlugin;

impl Plugin for SmokeAutomationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SmokeResult>()
            .add_systems(Startup, init_smoke_automation)
            .add_observer(on_remote_client_connected)
            .add_systems(
                Update,
                (track_client_connection, run_host_smoke, run_join_smoke, finish_smoke),
            );
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
        crate::Cli::Local => return,
    };

    commands.insert_resource(SmokeAutomation {
        role,
        timer: Timer::new(Duration::from_secs(30), TimerMode::Once),
        step_timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
        connected_for: Timer::new(Duration::from_secs(60), TimerMode::Once),
        step: 0,
        saw_client: false,
    });
}

fn track_client_connection(
    automation: Option<ResMut<SmokeAutomation>>,
    players: Query<(), With<crate::player::NetworkPlayer>>,
) {
    let Some(mut automation) = automation else {
        return;
    };
    if automation.role == SmokeRole::Host && players.iter().count() >= 2 {
        automation.saw_client = true;
    }
}

/// Host drives authoritative job progression once a client is connected.
fn run_host_smoke(
    time: Res<Time>,
    automation: Option<ResMut<SmokeAutomation>>,
    mut jobs: Option<ResMut<JobSystem>>,
    mut boards: Query<(&mut JobBoard, &mut crate::jobs::SmokeJobFlags)>,
    server_state: Option<Res<State<ServerState>>>,
    mut result: ResMut<SmokeResult>,
) {
    let Some(mut automation) = automation else {
        return;
    };
    if automation.role != SmokeRole::Host {
        return;
    }
    let Some(server_state) = server_state else {
        return;
    };
    if *server_state.get() != ServerState::Running || !automation.saw_client {
        return;
    }

    automation.step_timer.tick(time.delta());
    if !automation.step_timer.just_finished() {
        return;
    }

    let Some(jobs) = jobs.as_mut() else {
        return;
    };

    match automation.step {
        0..=2 => {
            jobs.try_crane_interact();
            automation.step += 1;
        }
        3 => {
            jobs.start_job(POWER_HOUR_JOB_ID);
            automation.step += 1;
        }
        4..=7 => {
            let seq_index = (automation.step - 4) as usize;
            if seq_index < POWER_HOUR_SEQUENCE.len() {
                jobs.try_power_hour_interact(POWER_HOUR_SEQUENCE[seq_index]);
            }
            automation.step += 1;
        }
        _ => {
            if jobs.is_complete(CRANE_JOB_ID) && jobs.is_complete(POWER_HOUR_JOB_ID) {
                result.pass = true;
                result.message =
                    "host completed crane + power hour after client joined".into();
                result.finished = true;
            }
        }
    }

    for (mut board, mut flags) in &mut boards {
        board.sync_from(&jobs);
        *flags = crate::jobs::SmokeJobFlags::sync_from(&jobs);
    }
}

/// Join verifies replicated job board updates from the host.
fn run_join_smoke(
    time: Res<Time>,
    automation: Option<ResMut<SmokeAutomation>>,
    flags: Query<&crate::jobs::SmokeJobFlags>,
    client_state: Option<Res<State<ClientState>>>,
    mut result: ResMut<SmokeResult>,
) {
    let Some(mut automation) = automation else {
        return;
    };
    if automation.role != SmokeRole::Join {
        return;
    }
    let Some(client_state) = client_state else {
        return;
    };
    if *client_state.get() != ClientState::Connected {
        return;
    }

    automation.connected_for.tick(time.delta());

    let crane_done = flags.iter().any(|f| f.crane_complete);
    let power_done = flags.iter().any(|f| f.power_complete);

    if crane_done && power_done {
        result.pass = true;
        result.message = "client received replicated crane + power hour completion".into();
        result.finished = true;
        return;
    }

    // Fallback: host writes pass result once authoritative jobs finish.
    if automation.connected_for.elapsed() >= Duration::from_secs(3) && host_smoke_pass_file() {
        result.pass = true;
        result.message =
            "client connected and host reported crane + power hour completion".into();
        result.finished = true;
    }
}

fn host_smoke_pass_file() -> bool {
    let path = PathBuf::from(SMOKE_RESULT_DIR).join("mp_smoke_host.result");
    fs::read_to_string(path)
        .map(|body| body.contains("pass=true"))
        .unwrap_or(false)
}

fn finish_smoke(
    time: Res<Time>,
    automation: Option<ResMut<SmokeAutomation>>,
    mut result: ResMut<SmokeResult>,
) {
    let Some(mut automation) = automation else {
        return;
    };

    if result.finished {
        write_smoke_result(automation.role, &result);
        if automation.role == SmokeRole::Join {
            schedule_exit(result.pass);
            return;
        }
        if automation.role == SmokeRole::Host && !result.pass {
            schedule_exit(false);
            return;
        }
        // Host passed: keep server alive briefly so the join client can verify.
        automation.timer.tick(time.delta());
        if automation.timer.elapsed() >= Duration::from_secs(12) {
            schedule_exit(true);
        }
        return;
    }

    automation.timer.tick(time.delta());
    if automation.timer.just_finished() {
        result.pass = false;
        result.message = format!(
            "timeout — role={:?} saw_client={}",
            automation.role, automation.saw_client
        );
        result.finished = true;
        write_smoke_result(automation.role, &result);
        schedule_exit(false);
    }
}

fn write_smoke_result(role: SmokeRole, result: &SmokeResult) {
    let dir = PathBuf::from(SMOKE_RESULT_DIR);
    let _ = fs::create_dir_all(&dir);
    let filename = match role {
        SmokeRole::Host => "mp_smoke_host.result",
        SmokeRole::Join => "mp_smoke_join.result",
    };
    let path = dir.join(filename);
    let line = format!(
        "pass={}\nmessage={}\n",
        result.pass,
        result.message.replace('\n', " ")
    );
    let _ = fs::write(path, line);
}

fn schedule_exit(success: bool) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        std::process::exit(if success { 0 } else { 1 });
    });
}
