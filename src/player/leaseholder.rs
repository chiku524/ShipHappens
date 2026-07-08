use bevy::prelude::*;

/// Phase 2 — Leaseholder sees full objectives; cannot interact (docs/ROOMS.md).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Leaseholder;
