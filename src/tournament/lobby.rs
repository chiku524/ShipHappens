//! Lobby ready-up — party hosts wait for Enter; local solo keeps short auto-start.

use bevy::prelude::*;

use crate::{
    tournament::{TournamentDirector, TournamentPhase},
    Cli,
};

#[derive(Resource, Debug, Clone)]
pub struct LobbyGate {
    /// When true, lobby timer does not expire until `host_ready`.
    pub require_ready: bool,
    pub host_ready: bool,
}

impl Default for LobbyGate {
    fn default() -> Self {
        Self {
            require_ready: false,
            host_ready: false,
        }
    }
}

pub fn configure_lobby_gate(mut gate: ResMut<LobbyGate>, cli: Res<Cli>) {
    // Host LAN parties: wait for Enter. Local / Join: auto timer.
    gate.require_ready = matches!(cli.as_ref(), Cli::Host { .. });
    gate.host_ready = false;
}

pub fn lobby_ready_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    director: Res<TournamentDirector>,
    mut gate: ResMut<LobbyGate>,
) {
    if director.phase != TournamentPhase::Lobby || !gate.require_ready {
        return;
    }
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::NumpadEnter) {
        gate.host_ready = true;
    }
}

/// Freeze lobby countdown until host ready when required.
pub fn apply_lobby_gate(gate: Res<LobbyGate>, mut director: ResMut<TournamentDirector>) {
    if director.phase != TournamentPhase::Lobby || !gate.require_ready || gate.host_ready {
        return;
    }
    if director.phase_timer < 1.0 {
        director.phase_timer = 1.0;
    }
}
