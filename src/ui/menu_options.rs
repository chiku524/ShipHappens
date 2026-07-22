//! Title-menu party options (slot size, timers, bracket, join target).

use std::net::{IpAddr, Ipv4Addr};

use bevy::prelude::*;

use crate::{
    core::DEFAULT_PORT,
    tournament::{types::SlotSize, DEFAULT_ONLINE_BRACKET_SIZE},
};

const JOIN_PRESETS: &[&str] = &["127.0.0.1", "192.168.0.1", "192.168.1.1", "10.0.0.1"];

#[derive(Resource, Debug, Clone)]
pub struct MenuPartyOptions {
    pub slot_size: SlotSize,
    pub production_timers: bool,
    /// Bracket team count (4 practice / 16 online).
    pub bracket_size: usize,
    pub dedicated_host: bool,
    pub join_preset: usize,
    pub join_port: u16,
}

impl Default for MenuPartyOptions {
    fn default() -> Self {
        Self {
            slot_size: SlotSize::Solo,
            production_timers: false,
            bracket_size: 4,
            dedicated_host: false,
            join_preset: 0,
            join_port: DEFAULT_PORT,
        }
    }
}

impl MenuPartyOptions {
    pub fn join_address(&self) -> IpAddr {
        let raw = JOIN_PRESETS[self.join_preset % JOIN_PRESETS.len()];
        raw.parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
    }

    pub fn join_host_label(&self) -> &'static str {
        JOIN_PRESETS[self.join_preset % JOIN_PRESETS.len()]
    }

    pub fn summary_line(&self) -> String {
        format!(
            "T team {} · B bracket {} · P timers {} · D dedicated {} · J join {}",
            self.slot_size.label(),
            self.bracket_size,
            if self.production_timers {
                "FULL"
            } else {
                "fast"
            },
            if self.dedicated_host { "ON" } else { "off" },
            self.join_host_label(),
        )
    }

    pub fn cycle_slot_size(&mut self) {
        self.slot_size = self.slot_size.cycle_next();
    }

    pub fn cycle_bracket(&mut self) {
        self.bracket_size = if self.bracket_size >= DEFAULT_ONLINE_BRACKET_SIZE {
            4
        } else if self.bracket_size >= 8 {
            DEFAULT_ONLINE_BRACKET_SIZE
        } else {
            8
        };
    }

    pub fn cycle_join_preset(&mut self) {
        self.join_preset = (self.join_preset + 1) % JOIN_PRESETS.len();
    }
}
