use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Homogeneous bracket slot size (see docs/TOURNAMENT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SlotSize {
    #[default]
    Solo = 1,
    Duo = 2,
    Trio = 3,
    Squad = 4,
}

impl SlotSize {
    pub fn player_count(self) -> usize {
        self as usize
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Solo => "Solo",
            Self::Duo => "Duo",
            Self::Trio => "Trio",
            Self::Squad => "Squad",
        }
    }

    pub fn cycle_next(self) -> Self {
        match self {
            Self::Solo => Self::Duo,
            Self::Duo => Self::Trio,
            Self::Trio => Self::Squad,
            Self::Squad => Self::Solo,
        }
    }
}

/// Map a network player seat index → bracket team index.
pub fn bracket_team_index(player_slot: u32, slot_size: SlotSize) -> u32 {
    player_slot / slot_size.player_count() as u32
}

/// Seat within a team (0 = first member).
pub fn seat_in_team(player_slot: u32, slot_size: SlotSize) -> u32 {
    player_slot % slot_size.player_count() as u32
}

/// Tournament lifecycle phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TournamentPhase {
    #[default]
    Lobby,
    RoomActive,
    Elimination,
    Finale,
    Podium,
    Complete,
}

/// Vault stage identifiers (docs/ROOMS.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RoomId {
    #[default]
    HrOrientation,
    CargoGantry,
    BreakerPanic,
    ShuttleMeltdown,
}

impl RoomId {
    pub fn label(self) -> &'static str {
        match self {
            Self::HrOrientation => "HR Orientation Bay",
            Self::CargoGantry => "Cargo Ring Gantry",
            Self::BreakerPanic => "Breaker Panic",
            Self::ShuttleMeltdown => "Shuttle Bay Meltdown",
        }
    }

    pub fn duration_secs(self, fast: bool) -> f32 {
        if fast {
            match self {
                Self::HrOrientation => 24.0,
                Self::CargoGantry => 35.0,
                Self::BreakerPanic => 35.0,
                Self::ShuttleMeltdown => 40.0,
            }
        } else {
            match self {
                Self::HrOrientation => 300.0,
                Self::CargoGantry => 360.0,
                Self::BreakerPanic => 360.0,
                Self::ShuttleMeltdown => 420.0,
            }
        }
    }

    pub fn sequence_index(self) -> usize {
        match self {
            Self::HrOrientation => 0,
            Self::CargoGantry => 1,
            Self::BreakerPanic => 2,
            Self::ShuttleMeltdown => 3,
        }
    }
}

/// Queue / match mode (Phase 5 extensions included).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MatchMode {
    #[default]
    PracticeVault,
    SquadRush,
    WagerIntern,
    WagerContractor,
    WagerExecutive,
    KingOfTheVault,
    FreeAgent,
}

impl MatchMode {
    pub fn uses_real_money(self) -> bool {
        matches!(
            self,
            Self::WagerIntern | Self::WagerContractor | Self::WagerExecutive
        )
    }

    pub fn buy_in_cents(self) -> u32 {
        match self {
            Self::PracticeVault | Self::SquadRush | Self::KingOfTheVault | Self::FreeAgent => 0,
            Self::WagerIntern => 100,
            Self::WagerContractor => 500,
            Self::WagerExecutive => 1000,
        }
    }
}

/// Unique bracket slot (solo player or team).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SlotId(pub u32);

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BracketSlot {
    pub id: SlotId,
    pub size: SlotSize,
    pub alive: bool,
    pub is_bot: bool,
    pub display_name: String,
    pub strikes: u8,
    pub leaseholder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoomProgress {
    pub cleared: bool,
    pub objective_count: u32,
    pub objective_target: u32,
    pub meltdown_meter: f32,
}
