use std::collections::HashMap;

use bevy::prelude::*;

use crate::tournament::types::RoomId;

/// Phase 3 — room mastery cosmetics.
#[derive(Resource, Debug, Default)]
pub struct RoomMastery {
    pub clears: HashMap<RoomId, u32>,
}

impl RoomMastery {
    pub fn record_clear(&mut self, room: RoomId) {
        *self.clears.entry(room).or_insert(0) += 1;
    }

    pub fn badge_for(&self, room: RoomId) -> Option<&'static str> {
        match self.clears.get(&room).copied().unwrap_or(0) {
            0 => None,
            1..=9 => Some("Intern"),
            10..=24 => Some("Contractor"),
            _ => Some("Executive"),
        }
    }
}

/// Phase 3 — seasonal room rotation stub.
#[derive(Resource, Debug, Clone)]
pub struct SeasonalVaultSet {
    pub season: u32,
    pub bonus_rooms: Vec<RoomId>,
}

impl Default for SeasonalVaultSet {
    fn default() -> Self {
        Self {
            season: 1,
            bonus_rooms: vec![RoomId::HrOrientation, RoomId::CargoGantry],
        }
    }
}

/// Phase 3 — spectator flag.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Spectator;

/// Phase 3 — Steam lobby placeholder.
#[derive(Resource, Debug, Default)]
pub struct SteamLobbyConfig {
    pub enabled: bool,
}

pub struct MetaPlugin;

impl Plugin for MetaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomMastery>()
            .init_resource::<SeasonalVaultSet>()
            .init_resource::<SteamLobbyConfig>();
    }
}
