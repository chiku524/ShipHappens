use bevy::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};

/// True on the dedicated/local authority (host server or offline `local` mode).
pub fn is_tournament_authority(
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
) -> bool {
    server.is_some() || client.is_none()
}
