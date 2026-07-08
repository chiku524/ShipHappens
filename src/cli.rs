use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;

use crate::core::DEFAULT_PORT;

/// Matches Godot smoke-test env var naming for future CI parity.
#[derive(Parser, PartialEq, Resource, Debug, Clone)]
#[command(name = "shiphappens", about = "ShipHappens Bevy migration spike")]
pub enum Cli {
    /// Offline greybox — movement + crane console without networking.
    #[command(name = "local")]
    Local,

    /// Host authoritative server (also plays locally).
    #[command(name = "host")]
    Host {
        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },

    /// Join an existing host.
    #[command(name = "join")]
    Join {
        #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::LOCALHOST))]
        address: IpAddr,

        #[arg(short, long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
}

impl Default for Cli {
    fn default() -> Self {
        if let Ok(role) = std::env::var("MP_TEST_ROLE") {
            return match role.to_ascii_lowercase().as_str() {
                "host" => Self::Host {
                    port: std::env::var("MP_TEST_PORT")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(DEFAULT_PORT),
                },
                "join" => Self::Join {
                    address: std::env::var("MP_TEST_ADDRESS")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                    port: std::env::var("MP_TEST_PORT")
                        .ok()
                        .and_then(|value| value.parse().ok())
                        .unwrap_or(DEFAULT_PORT),
                },
                _ => Self::parse(),
            };
        }

        Self::parse()
    }
}

impl Cli {
    pub fn is_online(&self) -> bool {
        !matches!(self, Self::Local)
    }
}

// Bevy Resource import for derive
use bevy::prelude::Resource;
