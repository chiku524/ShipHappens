//! Rematch / leave-to-Nest / disconnect / spectate for PudgyMon.

use bevy::prelude::*;
use bevy_replicon::prelude::*;

use crate::{
    flow::AppScreen,
    hub::ModeQueued,
    party::{HubReady, PartyDirector, PartyPhase, PartyPlan, PartySpawn},
    player::{LocalPlayer, NetworkPlayer, ThirdPersonCamera},
    settings::PauseState,
    world::MainCamera,
};

#[derive(Resource, Debug, Default)]
pub struct NetworkBanner {
    pub message: String,
    pub timer: f32,
}

impl NetworkBanner {
    pub fn show(&mut self, msg: impl Into<String>, secs: f32) {
        self.message = msg.into();
        self.timer = secs;
    }
}

#[derive(Component)]
pub struct Spectating;

#[derive(Resource, Debug, Default)]
pub struct SpectateTarget {
    pub slot: Option<u32>,
}

/// Set by Nest menu “Return to Nest”; consumed by session leave handler.
#[derive(Resource, Debug, Default)]
pub struct LeaveToNestRequest {
    pub pending: bool,
}

pub struct SessionFlowPlugin;

impl Plugin for SessionFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkBanner>()
            .init_resource::<SpectateTarget>()
            .init_resource::<LeaveToNestRequest>()
            .add_systems(
                Update,
                (
                    handle_rematch_and_leave,
                    tick_network_banner,
                    client_disconnect_flag,
                ),
            )
            .add_systems(
                Update,
                (
                    clear_spectate_on_phase_change,
                    cycle_spectate_target,
                    follow_spectate_camera,
                ),
            );
    }
}

fn handle_rematch_and_leave(
    keyboard: Res<ButtonInput<KeyCode>>,
    screen: Res<State<AppScreen>>,
    editor: Res<crate::hub::EditorMode>,
    mut pause: ResMut<PauseState>,
    mut camera: ResMut<ThirdPersonCamera>,
    mut director: ResMut<PartyDirector>,
    mut ready: ResMut<HubReady>,
    mut queued: ResMut<ModeQueued>,
    mut banner: ResMut<NetworkBanner>,
    mut leave: ResMut<LeaveToNestRequest>,
    spawn: Res<PartySpawn>,
    mut local: Query<&mut Transform, With<LocalPlayer>>,
    mut commands: Commands,
    server: Option<Res<bevy_replicon_renet::RenetServer>>,
    client: Option<Res<bevy_replicon_renet::RenetClient>>,
) {
    if *screen.get() != AppScreen::Playing || editor.active {
        leave.pending = false;
        return;
    }

    let is_authority = server.is_some() || client.is_none();
    let can_rematch = pause.paused
        || matches!(
            director.phase,
            PartyPhase::Results | PartyPhase::Hub
        );

    if keyboard.just_pressed(KeyCode::KeyR) && can_rematch {
        pause.paused = false;
        camera.captured = true;
        leave.pending = false;
        if is_authority {
            let plan = if director.plan == PartyPlan::Idle {
                PartyPlan::FullParty
            } else {
                director.plan
            };
            director.reset_party();
            ready.host_ready = false;
            queued.0 = Some(plan);
            banner.show("Rematch queued.", 3.0);
        } else {
            commands.client_trigger(crate::party::PartyClientCommand::Rematch);
            banner.show("Rematch requested…", 2.5);
        }
        return;
    }

    // Q or Nest menu — abort match / return to social plaza (no main menu).
    let leave_requested = keyboard.just_pressed(KeyCode::KeyQ) || leave.pending;
    if leave_requested {
        leave.pending = false;
        pause.paused = false;
        camera.captured = true;
        if is_authority {
            director.reset_party();
            ready.host_ready = false;
            queued.0 = None;
            if let Ok(mut tf) = local.single_mut() {
                tf.translation = spawn.hub;
            }
            banner.show("Back in The Nest.", 3.5);
        } else {
            commands.client_trigger(crate::party::PartyClientCommand::ReturnToHub);
            if let Ok(mut tf) = local.single_mut() {
                tf.translation = spawn.hub;
            }
            banner.show("Returning to The Nest…", 2.5);
        }
    }
}

fn tick_network_banner(time: Res<Time>, mut banner: ResMut<NetworkBanner>) {
    if banner.timer > 0.0 {
        banner.timer -= time.delta_secs();
        if banner.timer <= 0.0 {
            banner.message.clear();
        }
    }
}

fn client_disconnect_flag(
    client_state: Option<Res<State<ClientState>>>,
    screen: Res<State<AppScreen>>,
    mut pause: ResMut<PauseState>,
    mut camera: ResMut<ThirdPersonCamera>,
    mut banner: ResMut<NetworkBanner>,
    mut director: ResMut<PartyDirector>,
    mut ready: ResMut<HubReady>,
    mut queued: ResMut<ModeQueued>,
    mut last: Local<Option<ClientState>>,
) {
    let Some(state) = client_state else {
        return;
    };
    if *screen.get() != AppScreen::Playing {
        *last = Some(*state.get());
        return;
    }
    let current = *state.get();
    if let Some(prev) = *last {
        if prev == ClientState::Connected
            && matches!(current, ClientState::Disconnected | ClientState::Connecting)
        {
            pause.paused = false;
            camera.captured = true;
            director.reset_party();
            ready.host_ready = false;
            queued.0 = None;
            banner.show("Disconnected — back in The Nest (restart app for solo if needed)", 5.0);
        }
    }
    *last = Some(current);
}

fn clear_spectate_on_phase_change(
    director: Res<PartyDirector>,
    mut last: Local<Option<PartyPhase>>,
    spectating: Query<Entity, With<Spectating>>,
    mut target: ResMut<SpectateTarget>,
    mut commands: Commands,
) {
    let phase = director.phase;
    if *last != Some(phase) {
        for entity in &spectating {
            commands.entity(entity).remove::<Spectating>();
        }
        target.slot = None;
    }
    *last = Some(phase);
}

fn cycle_spectate_target(
    keyboard: Res<ButtonInput<KeyCode>>,
    screen: Res<State<AppScreen>>,
    local: Query<(), (With<LocalPlayer>, With<Spectating>)>,
    others: Query<&NetworkPlayer, Without<LocalPlayer>>,
    mut target: ResMut<SpectateTarget>,
    mut banner: ResMut<NetworkBanner>,
) {
    if *screen.get() != AppScreen::Playing || local.is_empty() {
        return;
    }
    let mut slots: Vec<u32> = others.iter().map(|n| n.slot).collect();
    slots.sort_unstable();
    if slots.is_empty() {
        return;
    }
    if target.slot.is_none() {
        target.slot = Some(slots[0]);
    }
    if keyboard.just_pressed(KeyCode::Tab) {
        let current = target.slot.unwrap_or(slots[0]);
        let idx = slots.iter().position(|s| *s == current).unwrap_or(0);
        let next = slots[(idx + 1) % slots.len()];
        target.slot = Some(next);
        banner.show(format!("Spectating slot {next}"), 2.0);
    }
}

fn follow_spectate_camera(
    time: Res<Time>,
    screen: Res<State<AppScreen>>,
    camera_state: Res<ThirdPersonCamera>,
    target: Res<SpectateTarget>,
    local_spec: Query<(), (With<LocalPlayer>, With<Spectating>)>,
    players: Query<(&NetworkPlayer, &Transform)>,
    mut camera: Query<&mut Transform, (With<MainCamera>, Without<NetworkPlayer>)>,
) {
    if *screen.get() != AppScreen::Playing || local_spec.is_empty() {
        return;
    }
    let Some(slot) = target.slot else {
        return;
    };
    let Some((_, player)) = players.iter().find(|(n, _)| n.slot == slot) else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let yaw = camera_state.yaw;
    let pitch = camera_state.pitch;
    let distance = camera_state.distance;
    let focus = player.translation + Vec3::Y * 1.15;
    let horizontal = distance * pitch.cos();
    let desired_eye = focus
        + Vec3::new(
            horizontal * yaw.sin(),
            -distance * pitch.sin(),
            horizontal * yaw.cos(),
        );
    let t = 1.0 - (-22.0 * time.delta_secs()).exp();
    camera_transform.translation = camera_transform.translation.lerp(desired_eye, t);
    camera_transform.look_at(focus, Vec3::Y);
}
