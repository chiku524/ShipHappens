pub mod ci;

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub use ci::ScoreAction;
use ci::{apply_action, composite_score, normalize_ci, CompositeInput, RawScoreSheet};

use crate::tournament::types::SlotId;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct PlayerScoreId(pub u32);

#[derive(Resource, Debug, Clone, Default)]
pub struct ScoringService {
    pub room_sheets: HashMap<PlayerScoreId, RawScoreSheet>,
    pub room_ci: HashMap<PlayerScoreId, f32>,
    pub slot_composite: HashMap<SlotId, f32>,
    pub slot_to_players: HashMap<SlotId, Vec<PlayerScoreId>>,
}

impl ScoringService {
    pub fn reset_room(&mut self) {
        self.room_sheets.clear();
        self.room_ci.clear();
    }

    /// Full wipe for rematch / leave (composites + slot maps).
    pub fn reset_tournament(&mut self) {
        self.room_sheets.clear();
        self.room_ci.clear();
        self.slot_composite.clear();
        self.slot_to_players.clear();
    }

    pub fn register_slot(&mut self, slot: SlotId, players: Vec<PlayerScoreId>) {
        self.slot_to_players.insert(slot.clone(), players.clone());
        for player in players {
            self.room_sheets.entry(player).or_default();
        }
    }

    pub fn record(&mut self, player: PlayerScoreId, action: ScoreAction) {
        let sheet = self.room_sheets.entry(player).or_default();
        if sheet.efficiency == 0.0 && action.efficiency_penalty() == 0.0 {
            sheet.efficiency = 100.0;
        }
        apply_action(sheet, action);
    }

    pub fn finalize_room(&mut self, clear_times: &HashMap<SlotId, (bool, f32)>) {
        let top = self.room_sheets.values().fold(RawScoreSheet::default(), |mut acc, s| {
            acc.objective = acc.objective.max(s.objective);
            acc.support = acc.support.max(s.support);
            acc.clutch = acc.clutch.max(s.clutch);
            acc
        });

        self.room_ci.clear();
        for (player, sheet) in &self.room_sheets {
            let ci = normalize_ci(*sheet, top);
            self.room_ci.insert(*player, ci.total);
        }

        let fastest = clear_times
            .values()
            .filter(|(cleared, _)| *cleared)
            .map(|(_, t)| *t)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0);

        for (slot, players) in &self.slot_to_players {
            let (cleared, time) = clear_times.get(slot).copied().unwrap_or((false, 999.0));
            let avg_eff = players
                .iter()
                .filter_map(|p| self.room_sheets.get(p))
                .map(|s| s.efficiency)
                .sum::<f32>()
                / players.len().max(1) as f32;
            let coop = players
                .iter()
                .filter_map(|p| self.room_sheets.get(p))
                .map(|s| s.support)
                .sum::<f32>()
                .min(100.0);
            let partial = players
                .iter()
                .filter_map(|p| self.room_sheets.get(p))
                .map(|s| s.objective)
                .sum::<f32>()
                .min(100.0);

            let composite = composite_score(CompositeInput {
                cleared,
                clear_time_secs: time,
                fastest_clear_secs: fastest,
                efficiency: avg_eff,
                cooperation: coop,
                partial_progress: partial,
            });
            self.slot_composite.insert(slot.clone(), composite);
        }
    }

    pub fn lowest_ci_player_in_slot(&self, slot: &SlotId) -> Option<PlayerScoreId> {
        let players = self.slot_to_players.get(slot)?;
        players
            .iter()
            .min_by(|a, b| {
                self.room_ci
                    .get(a)
                    .unwrap_or(&0.0)
                    .partial_cmp(self.room_ci.get(b).unwrap_or(&0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
}

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoringService>();
    }
}
