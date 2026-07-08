use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    core::{CRANE_JOB_ID, POWER_HOUR_JOB_ID, POWER_HOUR_SEQUENCE},
    interaction::InteractPrompt,
    jobs::{JobBoard, JobSystem},
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud_root)
            .add_systems(Update, update_hud_text);
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HudStatusText;

#[derive(Component)]
struct HudPromptText;

#[derive(Component)]
struct HudJobText;

fn spawn_hud_root(mut commands: Commands) {
    commands.spawn((
        HudRoot,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            ..Default::default()
        },
        children![
            (
                HudStatusText,
                Text::new("ShipHappens Bevy spike"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.9, 0.95, 1.0)),
            ),
            (
                HudPromptText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.85, 0.35)),
            ),
            (
                HudJobText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.75, 0.95, 0.75)),
            ),
        ],
    ));
}

fn update_hud_text(
    cli: Res<crate::Cli>,
    prompt: Res<InteractPrompt>,
    jobs: Option<Res<JobSystem>>,
    boards: Query<&JobBoard>,
    server_state: Res<State<ServerState>>,
    client_state: Res<State<ClientState>>,
    mut status: Query<
        &mut Text,
        (
            With<HudStatusText>,
            Without<HudPromptText>,
            Without<HudJobText>,
        ),
    >,
    mut prompts: Query<
        &mut Text,
        (
            With<HudPromptText>,
            Without<HudStatusText>,
            Without<HudJobText>,
        ),
    >,
    mut job_lines: Query<
        &mut Text,
        (
            With<HudJobText>,
            Without<HudStatusText>,
            Without<HudPromptText>,
        ),
    >,
) {
    let mode = match cli.as_ref() {
        crate::Cli::Local => "local",
        crate::Cli::Host { .. } => "host",
        crate::Cli::Join { .. } => "join",
    };

    if let Ok(mut text) = status.single_mut() {
        **text = format!(
            "ShipHappens Bevy spike — mode: {mode} | server: {:?} | client: {:?}",
            server_state.get(),
            client_state.get(),
        );
    }

    if let Ok(mut text) = prompts.single_mut() {
        **text = if prompt.message.is_empty() {
            prompt.last_action.clone()
        } else {
            format!("{}\n{}", prompt.message, prompt.last_action)
        };
    }

    let board = boards.iter().next();
    let crane = job_line(jobs.as_deref(), board, CRANE_JOB_ID, 3);
    let power = job_line(
        jobs.as_deref(),
        board,
        POWER_HOUR_JOB_ID,
        POWER_HOUR_SEQUENCE.len() as u32,
    );

    if let Ok(mut text) = job_lines.single_mut() {
        **text = format!("Crane of Regret: {crane}\nPower Hour: {power}");
    }
}

fn job_line(
    jobs: Option<&JobSystem>,
    board: Option<&JobBoard>,
    job_id: &str,
    target: u32,
) -> String {
    if let Some(jobs) = jobs {
        if jobs.is_complete(job_id) {
            return "complete".into();
        }
        let (current, target) = jobs.progress_for(job_id);
        if jobs.is_active(job_id) {
            return format!("{current}/{target}");
        }
        return "press F to start".into();
    }

    if let Some(board) = board {
        if let Some(state) = board.states.get(job_id) {
            if state.complete {
                return "complete".into();
            }
            if state.active {
                return format!("{}/{}", state.progress, target);
            }
        }
    }

    "idle".into()
}
