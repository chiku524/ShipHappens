use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::types::{BracketSlot, MatchMode, RoomId, SlotId, SlotSize, TournamentPhase};
use crate::scoring::ci::elimination_cut_count;

/// Dev-friendly defaults; production tournaments use 16 slots and full timers.
pub const DEFAULT_DEV_BRACKET_SIZE: usize = 4;
pub const DEFAULT_ONLINE_BRACKET_SIZE: usize = 16;
pub const LOBBY_DURATION_SECS: f32 = 3.0;
pub const ELIMINATION_DURATION_SECS: f32 = 2.0;
pub const PODIUM_DURATION_SECS: f32 = 5.0;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct TournamentConfig {
    pub slot_size: SlotSize,
    pub bracket_size: usize,
    pub match_mode: MatchMode,
    pub fast_timers: bool,
    pub human_slot: SlotId,
}

impl Default for TournamentConfig {
    fn default() -> Self {
        Self {
            slot_size: SlotSize::Solo,
            bracket_size: DEFAULT_DEV_BRACKET_SIZE,
            match_mode: MatchMode::PracticeVault,
            fast_timers: true,
            human_slot: SlotId(0),
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct TournamentDirector {
    pub phase: TournamentPhase,
    pub room: RoomId,
    pub room_index: usize,
    pub phase_timer: f32,
    pub slots: Vec<BracketSlot>,
    pub placements: Vec<SlotId>,
    pub danger_zone: Vec<SlotId>,
    pub remnant_hints: Vec<String>,
}

impl Default for TournamentDirector {
    fn default() -> Self {
        Self {
            phase: TournamentPhase::Lobby,
            room: RoomId::HrOrientation,
            room_index: 0,
            phase_timer: LOBBY_DURATION_SECS,
            slots: Vec::new(),
            placements: Vec::new(),
            danger_zone: Vec::new(),
            remnant_hints: Vec::new(),
        }
    }
}

impl TournamentDirector {
    pub fn bootstrap(config: &TournamentConfig) -> Self {
        let mut slots = Vec::with_capacity(config.bracket_size);
        for i in 0..config.bracket_size {
            let is_bot = SlotId(i as u32) != config.human_slot;
            slots.push(BracketSlot {
                id: SlotId(i as u32),
                size: config.slot_size,
                alive: true,
                is_bot,
                display_name: if is_bot {
                    format!("Bot-{i}")
                } else {
                    "You".into()
                },
                strikes: 0,
                leaseholder: false,
            });
        }

        Self {
            phase: TournamentPhase::Lobby,
            phase_timer: LOBBY_DURATION_SECS,
            slots,
            ..Default::default()
        }
    }

    pub fn alive_count(&self) -> usize {
        self.slots.iter().filter(|s| s.alive).count()
    }

    pub fn alive_slots(&self) -> Vec<&BracketSlot> {
        self.slots.iter().filter(|s| s.alive).collect()
    }

    pub fn room_for_index(index: usize) -> RoomId {
        match index {
            0 => RoomId::HrOrientation,
            1 => RoomId::CargoGantry,
            2 => RoomId::BreakerPanic,
            _ => RoomId::ShuttleMeltdown,
        }
    }

    pub fn apply_elimination(&mut self, composite: &[(SlotId, f32)], assign_strikes: bool) {
        let alive: Vec<_> = composite
            .iter()
            .filter(|(id, _)| self.slots.iter().any(|s| s.id == *id && s.alive))
            .collect();

        let cut = elimination_cut_count(alive.len(), self.room_index).min(alive.len());
        let mut sorted = alive;
        sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        self.danger_zone = sorted
            .iter()
            .take(cut)
            .map(|(id, _)| (*id).clone())
            .collect();

        let slot_size = self.slot_size();
        for (id, _) in sorted.iter().take(cut) {
            let Some(idx) = self.slots.iter().position(|s| s.id == *id) else {
                continue;
            };
            if slot_size == SlotSize::Solo || !assign_strikes {
                self.slots[idx].alive = false;
            } else {
                self.slots[idx].strikes = self.slots[idx].strikes.saturating_add(1);
                if self.slots[idx].strikes >= 2 {
                    self.slots[idx].alive = false;
                }
            }
            self.remnant_hints.push(format!("hint_for_{}", id.0));
        }
    }

    pub fn slot_size(&self) -> SlotSize {
        self.slots.first().map(|s| s.size).unwrap_or(SlotSize::Solo)
    }

    pub fn finalize_podium(&mut self, composite: &[(SlotId, f32)]) {
        let mut sorted = composite.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        self.placements = sorted.into_iter().map(|(id, _)| id).take(3).collect();
        self.phase = TournamentPhase::Podium;
        self.phase_timer = PODIUM_DURATION_SECS;
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, Default)]
pub struct TournamentSnapshot {
    pub phase: TournamentPhase,
    pub room: RoomId,
    pub alive_slots: u8,
    pub room_progress: u8,
    pub sort_target: u8,
    pub meltdown_percent: u8,
    pub announcer_line: String,
    pub tournament_complete: bool,
}
