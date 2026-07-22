pub mod layout;
pub mod spawner;

use bevy::prelude::*;

use crate::{
    player::Leaseholder,
    scoring::{PlayerScoreId, ScoringService, ScoreAction},
    tournament::types::{RoomId, RoomProgress, SlotId, SlotSize},
};

pub use layout::{
    relocate_players_on_room_enter, sync_room_layout, ActiveRoomLayout, LayoutMarkerId,
    RoomLayoutPiece, RoomSpawnPoint,
};

#[derive(Resource, Debug, Clone, Default)]
pub struct RoomRuntime {
    pub active: Option<RoomId>,
    pub progress: RoomProgress,
    pub slot_progress: std::collections::HashMap<SlotId, u32>,
    pub meltdown_rate: f32,
    pub sort_target: u8,
    /// Finale failed because meltdown hit 100.
    pub failed: bool,
    bot_entropy: u32,
}

impl RoomRuntime {
    pub fn begin(&mut self, room: RoomId, alive_slots: usize, slot_size: SlotSize) {
        self.active = Some(room);
        self.slot_progress.clear();
        let per_slot = scaled_target(room, slot_size);
        for i in 0..alive_slots {
            self.slot_progress.insert(SlotId(i as u32), 0);
        }
        self.progress = RoomProgress {
            cleared: false,
            objective_count: 0,
            objective_target: per_slot.saturating_mul(alive_slots as u32),
            meltdown_meter: 0.0,
        };
        self.meltdown_rate = if room == RoomId::ShuttleMeltdown {
            2.0
        } else {
            0.0
        };
        self.sort_target = 0;
        self.bot_entropy = 1;
        self.failed = false;
    }

    pub fn progress_percent(&self) -> u8 {
        if self.progress.objective_target == 0 {
            return 0;
        }
        ((self.progress.objective_count as f32 / self.progress.objective_target as f32) * 100.0)
            .min(100.0) as u8
    }

    /// Ambient meltdown rise + bot pacing. Returns true if the vault just failed.
    pub fn tick_ambient(&mut self, dt: f32) -> bool {
        if self.progress.cleared || self.failed || self.meltdown_rate <= 0.0 {
            return false;
        }
        // Baseline rise independent of player actions (GDD: +2/s feel, scaled for fast timers).
        self.progress.meltdown_meter =
            (self.progress.meltdown_meter + self.meltdown_rate * dt).min(100.0);
        if self.progress.meltdown_meter >= 100.0 {
            self.failed = true;
            return true;
        }
        false
    }

    pub fn tick_bot_slot(&mut self, scoring: &mut ScoringService, slot: &SlotId, dt: f32) {
        if self.progress.cleared || self.failed {
            return;
        }
        self.bot_entropy = self.bot_entropy.wrapping_add(1);
        // Slower than before so the human can still move the needle.
        let chance = dt * 0.12;
        if pseudo_rand(self.bot_entropy) < chance {
            self.advance_slot(scoring, slot, true);
        }
    }

    pub fn sort_target_label(&self) -> &'static str {
        Self::sort_label(self.sort_target)
    }

    pub fn sort_label(target: u8) -> &'static str {
        match target {
            0 => "Hot Dogs",
            1 => "Toasters",
            2 => "Premium Air",
            _ => "Write-Ups",
        }
    }

    pub fn meltdown_percent(&self) -> u8 {
        self.progress.meltdown_meter.min(100.0) as u8
    }

    pub fn player_action(
        &mut self,
        scoring: &mut ScoringService,
        player: PlayerScoreId,
        slot: &SlotId,
        action: ScoreAction,
    ) {
        if self.failed {
            return;
        }
        scoring.record(player, action);

        // Coolant valves fight the meltdown meter (GDD).
        if matches!(action, ScoreAction::CoolantValve) {
            self.progress.meltdown_meter = (self.progress.meltdown_meter - 12.0).max(0.0);
        }

        if action.objective_delta() > 0.0 {
            let entry = self.slot_progress.entry(slot.clone()).or_insert(0);
            *entry += 1;
            self.progress.objective_count += 1;
            // Wrong inputs / mistakes spike meltdown in the finale.
            if self.meltdown_rate > 0.0 && matches!(action, ScoreAction::IncorrectSort | ScoreAction::BreakerWrong) {
                self.progress.meltdown_meter =
                    (self.progress.meltdown_meter + 15.0).min(100.0);
            }
            if self.progress.objective_count >= self.progress.objective_target {
                self.progress.cleared = true;
                scoring.record(player, ScoreAction::RoomClearBonus);
            }
        }
    }

    fn advance_slot(&mut self, scoring: &mut ScoringService, slot: &SlotId, positive: bool) {
        if self.failed {
            return;
        }
        let room = self.active.unwrap_or(RoomId::HrOrientation);
        let entry = self.slot_progress.entry(slot.clone()).or_insert(0);
        let player = PlayerScoreId(slot.0 * 10);

        if positive {
            *entry += 1;
            self.progress.objective_count += 1;
            let action = match room {
                RoomId::HrOrientation => ScoreAction::CorrectSort,
                RoomId::CargoGantry => ScoreAction::CrateDelivered,
                RoomId::BreakerPanic => ScoreAction::BreakerCorrect,
                RoomId::ShuttleMeltdown => ScoreAction::CoolantValve,
            };
            scoring.record(player, action);
            if matches!(action, ScoreAction::CoolantValve) {
                self.progress.meltdown_meter = (self.progress.meltdown_meter - 12.0).max(0.0);
            }
        } else {
            scoring.record(player, ScoreAction::IncorrectSort);
            if self.meltdown_rate > 0.0 {
                self.progress.meltdown_meter =
                    (self.progress.meltdown_meter + 15.0).min(100.0);
            }
        }

        if self.progress.objective_count >= self.progress.objective_target {
            self.progress.cleared = true;
            scoring.record(player, ScoreAction::RoomClearBonus);
        }
    }

    pub fn finish(
        &mut self,
        slot_size: SlotSize,
    ) -> std::collections::HashMap<SlotId, (bool, f32)> {
        let room = self.active.unwrap_or(RoomId::HrOrientation);
        let target = scaled_target(room, slot_size);
        let mut out = std::collections::HashMap::new();
        for (slot, count) in &self.slot_progress {
            let cleared = *count >= target;
            let time = if cleared {
                30.0
            } else {
                999.0
            };
            out.insert(slot.clone(), (cleared, time));
        }
        self.active = None;
        out
    }
}

fn scaled_target(room: RoomId, slot_size: SlotSize) -> u32 {
    let base = match room {
        RoomId::HrOrientation => 6,
        RoomId::CargoGantry => 4,
        RoomId::BreakerPanic => 4,
        RoomId::ShuttleMeltdown => 6,
    };
    base * slot_size.player_count() as u32
}

fn pseudo_rand(seed: u32) -> f32 {
    let x = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    (x % 10_000) as f32 / 10_000.0
}

pub struct RoomsPlugin;

impl Plugin for RoomsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoomRuntime>()
            .init_resource::<ActiveRoomLayout>()
            .init_resource::<RoomSpawnPoint>()
            .add_systems(
                Update,
                (sync_room_layout, relocate_players_on_room_enter).chain(),
            );
    }
}

/// Loads vault room layouts + arena shell; call from app startup before arena/player spawn.
pub fn load_room_layouts(mut commands: Commands) {
    let base = format!("{}/data/rooms", env!("CARGO_MANIFEST_DIR"));
    let catalog = crate::data::RoomLayoutCatalog::load_from_dir(&base)
        .unwrap_or_else(|err| panic!("failed to load room layouts: {err}"));
    info!("loaded {} vault room layouts from data/rooms", catalog.len());
    commands.insert_resource(catalog);

    let arena_path = format!("{base}/arena.json");
    let arena = crate::data::load_arena_layout(&arena_path)
        .unwrap_or_else(|err| panic!("failed to load arena layout: {err}"));
    let lobby = Vec3::from_array(arena.lobby_spawn);
    commands.insert_resource(crate::data::ArenaLayout(arena));
    commands.insert_resource(RoomSpawnPoint {
        lobby,
        current: lobby,
    });
}

/// One leaseholder per alive team that has 2+ humans (seat rotates by room).
/// Solo / underfilled teams keep hands — no forced leaseholder.
pub fn assign_leaseholder(
    mut director: ResMut<crate::tournament::TournamentDirector>,
    mut commands: Commands,
    players: Query<(Entity, &crate::player::NetworkPlayer)>,
) {
    let slot_size = director.slot_size();
    for slot in director.slots.iter_mut() {
        slot.leaseholder = false;
    }
    if slot_size == SlotSize::Solo {
        for (entity, _) in &players {
            commands.entity(entity).remove::<Leaseholder>();
        }
        return;
    }

    let seat_pick = director.room_index as u32 % slot_size.player_count() as u32;
    let mut team_counts = [0u32; 32];
    for (_, net) in &players {
        let team = crate::tournament::types::bracket_team_index(net.slot, slot_size) as usize;
        if team < team_counts.len() {
            team_counts[team] += 1;
        }
    }

    for (entity, net) in &players {
        let team = crate::tournament::types::bracket_team_index(net.slot, slot_size);
        let seat = crate::tournament::types::seat_in_team(net.slot, slot_size);
        let team_alive = director
            .slots
            .iter()
            .find(|s| s.id.0 == team)
            .map(|s| s.alive)
            .unwrap_or(false);
        let filled = team_counts
            .get(team as usize)
            .copied()
            .unwrap_or(0)
            >= 2;
        if team_alive && filled && seat == seat_pick {
            if let Some(slot) = director.slots.iter_mut().find(|s| s.id.0 == team) {
                slot.leaseholder = true;
            }
            commands.entity(entity).insert(Leaseholder);
        } else {
            commands.entity(entity).remove::<Leaseholder>();
        }
    }
}
