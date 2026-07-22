//! Season points ledger (off-chain authoritative for MVP).

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cosmetics::CosmeticsCatalog;

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SeasonLedger {
    pub season_id: String,
    pub points: u32,
    pub parties_played: u32,
    pub unlocked: Vec<String>,
}

impl Default for SeasonLedger {
    fn default() -> Self {
        Self {
            season_id: "s1".into(),
            points: 0,
            parties_played: 0,
            unlocked: vec!["skin_starter".into()],
        }
    }
}

impl SeasonLedger {
    pub fn add_points(&mut self, amount: u32) {
        self.points = self.points.saturating_add(amount);
        self.parties_played = self.parties_played.saturating_add(1);
    }

    pub fn path() -> PathBuf {
        if let Ok(base) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(base)
                .join(crate::brand::APP_DATA_DIR)
                .join("season.json")
        } else {
            PathBuf::from("season.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::path();
        let Ok(mut file) = File::open(&path) else {
            return Self::default();
        };
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_err() {
            return Self::default();
        }
        serde_json::from_str(&buf).unwrap_or_default()
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Ok(mut file) = File::create(&path) {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }

    pub fn sync_unlocks(&mut self, catalog: &CosmeticsCatalog) {
        for item in &catalog.items {
            if self.points >= item.cost_points && !self.unlocked.contains(&item.id) {
                self.unlocked.push(item.id.clone());
            }
        }
    }
}

pub struct SeasonPlugin;

impl Plugin for SeasonPlugin {
    fn build(&self, app: &mut App) {
        let ledger = SeasonLedger::load();
        app.insert_resource(ledger)
            .add_systems(Update, persist_season_unlocks);
    }
}

fn persist_season_unlocks(
    mut ledger: ResMut<SeasonLedger>,
    catalog: Res<CosmeticsCatalog>,
    mut dirty: Local<u32>,
) {
    if !ledger.is_changed() && *dirty == ledger.points {
        return;
    }
    ledger.sync_unlocks(&catalog);
    ledger.save();
    *dirty = ledger.points;
}
