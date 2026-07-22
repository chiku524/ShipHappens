//! Nest / party roster + stage progression authority.

use bevy::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};
use serde::{Deserialize, Serialize};

pub mod director;
pub mod net;

pub use director::{
    HubReady, PartyDirector, PartyPhase, PartyPlan, PartyPlugin, PartySnapshot, StageKind,
};
pub use net::{PartyClientCommand, PartyNetPlugin};

pub fn is_party_authority(
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
) -> bool {
    server.is_some() || client.is_none()
}

#[derive(Resource, Debug, Clone)]
pub struct PartySpawn {
    pub hub: Vec3,
}

impl Default for PartySpawn {
    fn default() -> Self {
        Self {
            hub: Vec3::new(0.0, 1.0, 14.0),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct PartyConfig {
    pub bot_fill: usize,
    pub max_party: usize,
}

impl Default for PartyConfig {
    fn default() -> Self {
        Self {
            bot_fill: 4,
            max_party: 8,
        }
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PartyBot {
    pub slot: u32,
}
