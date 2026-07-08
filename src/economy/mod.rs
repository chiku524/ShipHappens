use bevy::prelude::*;

use crate::tournament::types::{MatchMode, SlotId};

/// Practice Vault Credits (Phase 1). Real wallet gated in Phase 4.
#[derive(Resource, Debug, Clone, Default)]
pub struct PracticeLedger {
    pub balance_vc: u32,
    pub last_payouts_vc: Vec<(SlotId, u32)>,
    pub tournaments_played: u32,
}

impl PracticeLedger {
    pub fn reset_for_tournament(&mut self, mode: MatchMode, slot_count: usize) {
        self.tournaments_played += 1;
        let _ = (mode, slot_count);
    }

    pub fn accrue_practice_rewards(&mut self, _placements: &[SlotId]) {
        self.balance_vc += 5;
    }

    pub fn apply_podium(&mut self, payouts: [(u32, SlotId); 3], placements: &[SlotId]) {
        self.last_payouts_vc.clear();
        for (amount, slot) in payouts {
            if placements.iter().any(|p| p.0 == slot.0) {
                self.last_payouts_vc.push((slot, amount));
                self.balance_vc += amount;
            }
        }
    }
}

pub struct PayoutCalculator;

impl PayoutCalculator {
    pub fn top_three(mode: MatchMode, slots: usize, players_per_slot: usize) -> [(u32, SlotId); 3] {
        let buy_in = mode.buy_in_cents();
        let gross = buy_in * slots as u32 * players_per_slot as u32;
        let pool = gross * 95 / 100;
        let (first, second, third) = crate::scoring::ci::payout_split_cents(pool);
        [
            (first / 100, SlotId(0)),
            (second / 100, SlotId(1)),
            (third / 100, SlotId(2)),
        ]
    }
}

/// Phase 4 — real-money wallet (gated).
#[derive(Resource, Debug, Clone, Default)]
pub struct Wallet {
    pub balance_cents: u32,
    pub weekly_deposited_cents: u32,
    pub weekly_lost_cents: u32,
    pub age_verified: bool,
    pub practice_games: u32,
}

impl Wallet {
    pub const DEPOSIT_CAP_CENTS: u32 = 1000;
    pub const WEEKLY_DEPOSIT_CAP_CENTS: u32 = 1000;
    pub const WEEKLY_LOSS_CAP_CENTS: u32 = 2500;
    pub const MIN_PRACTICE_GAMES: u32 = 20;

    pub fn can_enter_wager(&self, mode: MatchMode) -> bool {
        if !mode.uses_real_money() {
            return true;
        }
        self.age_verified && self.practice_games >= Self::MIN_PRACTICE_GAMES
    }

    pub fn try_deposit(&mut self, cents: u32) -> bool {
        if self.weekly_deposited_cents + cents > Self::WEEKLY_DEPOSIT_CAP_CENTS {
            return false;
        }
        if self.balance_cents + cents > Self::DEPOSIT_CAP_CENTS {
            return false;
        }
        self.balance_cents += cents;
        self.weekly_deposited_cents += cents;
        true
    }
}

#[derive(Resource, Debug, Default)]
pub struct WagerGate;

impl WagerGate {
    pub fn check(wallet: &Wallet, mode: MatchMode) -> Result<(), &'static str> {
        if !mode.uses_real_money() {
            return Ok(());
        }
        if !wallet.age_verified {
            return Err("age verification required");
        }
        if wallet.practice_games < Wallet::MIN_PRACTICE_GAMES {
            return Err("complete 20 practice tournaments first");
        }
        if wallet.weekly_lost_cents >= Wallet::WEEKLY_LOSS_CAP_CENTS {
            return Err("weekly loss limit reached");
        }
        Ok(())
    }
}

/// Phase 4 — audit trail stub.
#[derive(Resource, Debug, Default)]
pub struct AuditLog {
    pub entries: Vec<String>,
}

impl AuditLog {
    pub fn record(&mut self, message: impl Into<String>) {
        self.entries.push(message.into());
    }
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PracticeLedger>()
            .init_resource::<Wallet>()
            .init_resource::<WagerGate>()
            .init_resource::<AuditLog>();
    }
}
