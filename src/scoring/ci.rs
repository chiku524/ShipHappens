//! Contribution Index and composite scoring (docs/SCORING.md).

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RawScoreSheet {
    pub objective: f32,
    pub support: f32,
    pub efficiency: f32,
    pub clutch: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct NormalizedCi {
    pub objective: f32,
    pub support: f32,
    pub efficiency: f32,
    pub clutch: f32,
    pub total: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreAction {
    CorrectSort,
    IncorrectSort,
    CrateDelivered,
    BreakerCorrect,
    BreakerWrong,
    CoolantValve,
    DoorSealed,
    EscapeCrate,
    SubTaskComplete,
    RoomClearBonus,
    Revive,
    DoorHold,
    CarryForTeammate,
    CorrectPing,
    LeaseCallout,
    CatchDrop,
    FallBonk,
    DroppedCarryable,
    WrongInput,
    Idle,
    Grief,
    WriteUpTier,
    SuddenDeathLoss,
    LastSecondClear,
    SaveSubTask,
    SuddenDeathWin,
    CleanRoom,
}

impl ScoreAction {
    pub fn objective_delta(self) -> f32 {
        match self {
            Self::CorrectSort => 8.0,
            Self::IncorrectSort => -3.0,
            Self::CrateDelivered => 15.0,
            Self::BreakerCorrect => 12.0,
            Self::BreakerWrong => -8.0,
            Self::CoolantValve => 10.0,
            Self::DoorSealed => 15.0,
            Self::EscapeCrate => 20.0,
            Self::SubTaskComplete => 25.0,
            Self::RoomClearBonus => 50.0,
            _ => 0.0,
        }
    }

    pub fn support_delta(self) -> f32 {
        match self {
            Self::Revive => 20.0,
            Self::DoorHold => 5.0,
            Self::CarryForTeammate => 10.0,
            Self::CorrectPing => 3.0,
            Self::LeaseCallout => 8.0,
            Self::CatchDrop => 12.0,
            _ => 0.0,
        }
    }

    pub fn efficiency_penalty(self) -> f32 {
        match self {
            Self::FallBonk => 5.0,
            Self::DroppedCarryable => 8.0,
            Self::WrongInput => 4.0,
            Self::Idle => 20.0,
            Self::Grief => 50.0,
            Self::WriteUpTier => 10.0,
            Self::SuddenDeathLoss => 30.0,
            _ => 0.0,
        }
    }

    pub fn clutch_delta(self) -> f32 {
        match self {
            Self::LastSecondClear => 15.0,
            Self::SaveSubTask => 20.0,
            Self::SuddenDeathWin => 30.0,
            Self::CleanRoom => 10.0,
            _ => 0.0,
        }
    }
}

pub fn apply_action(sheet: &mut RawScoreSheet, action: ScoreAction) {
    sheet.objective += action.objective_delta();
    sheet.support += action.support_delta();
    sheet.efficiency = (sheet.efficiency - action.efficiency_penalty()).max(0.0);
    sheet.clutch = (sheet.clutch + action.clutch_delta()).min(30.0);
}

pub fn normalize_ci(sheet: RawScoreSheet, slot_top: RawScoreSheet) -> NormalizedCi {
    let norm = |value: f32, top: f32| {
        if top <= f32::EPSILON {
            0.0
        } else {
            (value / top * 100.0).clamp(0.0, 100.0)
        }
    };

    let objective = norm(sheet.objective.max(0.0), slot_top.objective.max(0.0));
    let support = norm(sheet.support, slot_top.support);
    let efficiency = sheet.efficiency.clamp(0.0, 100.0);
    let clutch = norm(sheet.clutch, slot_top.clutch.max(1.0));

    let total = objective * 0.45 + support * 0.25 + efficiency * 0.20 + clutch * 0.10;

    NormalizedCi {
        objective,
        support,
        efficiency,
        clutch,
        total,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CompositeInput {
    pub cleared: bool,
    pub clear_time_secs: f32,
    pub fastest_clear_secs: f32,
    pub efficiency: f32,
    pub cooperation: f32,
    pub partial_progress: f32,
}

pub fn composite_score(input: CompositeInput) -> f32 {
    let cleared_pts = if input.cleared { 100.0 } else { 0.0 };
    let speed = if input.cleared && input.clear_time_secs > f32::EPSILON {
        (100.0 * (input.fastest_clear_secs / input.clear_time_secs)).min(100.0)
    } else {
        input.partial_progress.clamp(0.0, 50.0)
    };

    cleared_pts * 0.40 + speed * 0.25 + input.efficiency.clamp(0.0, 100.0) * 0.20
        + input.cooperation.clamp(0.0, 100.0) * 0.15
}

pub fn elimination_cut_count(remaining: usize, room_index: usize) -> usize {
    match room_index {
        0 => remaining / 4,
        1 => remaining / 3,
        2 => remaining / 2,
        _ => 0,
    }
    .max(1)
}

pub fn payout_split_cents(pool_cents: u32) -> (u32, u32, u32) {
    let first = pool_cents * 50 / 100;
    let second = pool_cents * 30 / 100;
    let third = pool_cents.saturating_sub(first + second);
    (first, second, third)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ci_weights_match_design_doc() {
        let sheet = RawScoreSheet {
            objective: 100.0,
            support: 50.0,
            efficiency: 80.0,
            clutch: 20.0,
        };
        let top = RawScoreSheet {
            objective: 100.0,
            support: 50.0,
            efficiency: 100.0,
            clutch: 20.0,
        };
        let ci = normalize_ci(sheet, top);
        assert!((ci.total - (100.0 * 0.45 + 100.0 * 0.25 + 80.0 * 0.20 + 100.0 * 0.10)).abs() < 0.01);
    }

    #[test]
    fn payout_splits_95_percent_pool() {
        let (a, b, c) = payout_split_cents(7600);
        assert_eq!(a + b + c, 7600);
        assert_eq!(a, 3800);
        assert_eq!(b, 2280);
        assert_eq!(c, 1520);
    }
}
