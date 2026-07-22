//! Product naming — see docs/BRAND.md.

/// Universe / IP line.
pub const UNIVERSE: &str = "Pugdy Monsters";

/// Full game title.
pub const TITLE: &str = "PugdyMon: Party Saga";

/// Short product name (window chrome, folders).
pub const SHORT: &str = "PugdyMon";

/// Full three-stage circuit label.
pub const PARTY_SAGA: &str = "Party Saga";

/// Social hub label.
pub const NEST: &str = "The Nest";

/// Local app data folder under LOCALAPPDATA.
pub const APP_DATA_DIR: &str = "PugdyMon";

/// Window title when playing.
pub fn window_title() -> String {
    TITLE.to_string()
}

/// Window title for smoke.
pub fn smoke_window_title() -> String {
    format!("{SHORT} Smoke")
}
