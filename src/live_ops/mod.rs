use bevy::prelude::*;

use crate::tournament::types::SlotId;

/// Phase 5 — seasonal leaderboard stub.
#[derive(Resource, Debug, Default)]
pub struct Leaderboard {
    pub entries: Vec<(String, u32)>,
}

impl Leaderboard {
    pub fn submit(&mut self, name: impl Into<String>, score: u32) {
        self.entries.push((name.into(), score));
        self.entries.sort_by(|a, b| b.1.cmp(&a.1));
        self.entries.truncate(100);
    }
}

/// Phase 5 — handshake side bet between two slots.
#[derive(Debug, Clone)]
pub struct HandshakeWager {
    pub slot_a: SlotId,
    pub slot_b: SlotId,
    pub percent_of_payout: u8,
    pub accepted: bool,
}

#[derive(Resource, Debug, Default)]
pub struct SideBetBoard {
    pub pending: Vec<HandshakeWager>,
}

/// Phase 5 — King of the Vault queue state.
#[derive(Resource, Debug, Default)]
pub struct KingOfTheVaultState {
    pub reigning_slot: Option<SlotId>,
    pub win_streak: u32,
}

/// Phase 5 — remnant hints left by eliminated slots.
#[derive(Resource, Debug, Default)]
pub struct RemnantClueBoard {
    pub hints: Vec<String>,
}

pub struct LiveOpsPlugin;

impl Plugin for LiveOpsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Leaderboard>()
            .init_resource::<SideBetBoard>()
            .init_resource::<KingOfTheVaultState>()
            .init_resource::<RemnantClueBoard>();
    }
}
