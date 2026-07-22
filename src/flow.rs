//! App-level screens — interactive local play starts on Title (auth), then Nest.

use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum AppScreen {
    /// Sign in / register intro (interactive local default).
    #[default]
    Title,
    /// Social hub + matches after auth (or skip for smoke / host / join).
    Playing,
}

/// Skip the auth intro for network roles, smoke, and explicit bypass.
pub fn should_skip_title(cli: &crate::Cli) -> bool {
    matches!(cli, crate::Cli::Host { .. } | crate::Cli::Join { .. })
        || std::env::var("MP_TEST_ROLE").is_ok()
        || std::env::var("PUDGYMON_SKIP_AUTH").ok().is_some_and(|v| {
            let v = v.to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        })
}

/// Initial screen for this process (headless/smoke never wait on auth UI).
pub fn initial_screen(headless: bool, enable_smoke: bool, cli: &crate::Cli) -> AppScreen {
    if headless || enable_smoke || should_skip_title(cli) {
        AppScreen::Playing
    } else {
        AppScreen::Title
    }
}
