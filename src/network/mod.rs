use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{
    netcode::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    renet::ConnectionConfig,
    RenetChannelsExt, RenetClient, RenetServer,
};

use crate::{
    cli::Cli,
    core::{MAX_PLAYERS, PROTOCOL_ID},
    jobs::{JobBoard, JobSystem},
    player::{NetworkPlayer, PlayerColor, PlayerName, PlayerRegistry},
    world::GameplayEntity,
};

#[derive(Resource, Default)]
pub struct PlayerSlotCounter(u32);

impl PlayerSlotCounter {
    pub fn next(&mut self) -> u32 {
        let slot = self.0;
        self.0 += 1;
        slot
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerRegistry>()
            .init_resource::<PlayerSlotCounter>()
            .replicate::<Transform>()
            .replicate::<NetworkPlayer>()
            .replicate::<PlayerName>()
            .replicate::<PlayerColor>()
            .replicate::<JobBoard>()
            .replicate::<crate::jobs::SmokeJobFlags>()
            .add_client_event::<crate::player::MoveInput>(Channel::Unordered)
            .add_observer(spawn_player_for_client)
            .add_observer(despawn_player_for_client)
            .add_observer(crate::player::apply_move_input)
            .add_systems(Update, sync_job_board_from_resource);
    }
}

pub fn init_network_backend(
    mut commands: Commands,
    cli: Res<Cli>,
    channels: Res<RepliconChannels>,
) -> Result {
    match cli.clone() {
        Cli::Local => {
            info!("offline greybox — no network backend");
        }
        Cli::Host { port, .. } => {
            info!("hosting on port {port}");
            let server_channels_config = channels.server_configs();
            let client_channels_config = channels.client_configs();

            let server = RenetServer::new(ConnectionConfig {
                server_channels_config,
                client_channels_config,
                ..Default::default()
            });

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, port))?;
            let server_config = ServerConfig {
                current_time,
                max_clients: MAX_PLAYERS,
                protocol_id: PROTOCOL_ID,
                authentication: ServerAuthentication::Unsecure,
                public_addresses: Default::default(),
            };
            let transport = NetcodeServerTransport::new(server_config, socket)?;

            commands.insert_resource(server);
            commands.insert_resource(transport);
        }
        Cli::Join { address, port } => {
            info!("joining {address}:{port}");
            let server_channels_config = channels.server_configs();
            let client_channels_config = channels.client_configs();

            let client = RenetClient::new(ConnectionConfig {
                server_channels_config,
                client_channels_config,
                ..Default::default()
            });

            let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
            let client_id = current_time.as_millis() as u64;
            let server_addr = SocketAddr::new(address, port);
            let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
            let authentication = ClientAuthentication::Unsecure {
                client_id,
                protocol_id: PROTOCOL_ID,
                server_addr,
                user_data: None,
            };
            let transport = NetcodeClientTransport::new(current_time, authentication, socket)?;

            commands.insert_resource(client);
            commands.insert_resource(transport);
        }
    }

    Ok(())
}

pub fn spawn_offline_player(
    mut commands: Commands,
    cli: Res<Cli>,
    mut registry: ResMut<PlayerRegistry>,
    spawn_point: Res<crate::rooms::RoomSpawnPoint>,
) {
    if !matches!(*cli, Cli::Local) {
        return;
    }

    let player = spawn_player_entity(&mut commands, 0, spawn_point.lobby);
    registry.local_player = Some(player);
    commands.entity(player).insert(crate::player::LocalPlayer);
}

fn spawn_player_for_client(
    add: On<Add, ConnectedClient>,
    mut commands: Commands,
    mut slots: ResMut<PlayerSlotCounter>,
) {
    let slot = slots.next();
    let position = Vec3::new((slot as f32) * 4.0, 1.0, 8.0);
    let player = spawn_player_entity(&mut commands, slot, position);
    commands.entity(add.entity).insert(OwnedPlayer(player));
}

/// Links a connected client entity to its spawned player on the server.
#[derive(Component, Clone, Copy, Debug)]
pub struct OwnedPlayer(pub Entity);

fn despawn_player_for_client(
    remove: On<Remove, ConnectedClient>,
    mut commands: Commands,
    mut registry: ResMut<PlayerRegistry>,
    owners: Query<&OwnedPlayer>,
) {
    if let Ok(owned) = owners.get(remove.entity) {
        let player = owned.0;
        registry.players.remove(&remove.entity);
        if registry.local_player == Some(player) {
            registry.local_player = None;
        }
        commands.entity(player).despawn();
    }
}

fn spawn_player_entity(commands: &mut Commands, slot: u32, position: Vec3) -> Entity {
    let color = player_color_for_slot(slot);
    commands
        .spawn((
            GameplayEntity,
            NetworkPlayer { slot },
            PlayerName(format!("Player{slot}")),
            PlayerColor(color),
            Transform::from_translation(position),
            Replicated,
            Visibility::default(),
        ))
        .id()
}

fn player_color_for_slot(slot: u32) -> [f32; 3] {
    const PALETTE: [[f32; 3]; 8] = [
        [0.95, 0.35, 0.35],
        [0.35, 0.75, 0.95],
        [0.45, 0.90, 0.45],
        [0.95, 0.85, 0.35],
        [0.80, 0.45, 0.95],
        [0.95, 0.55, 0.25],
        [0.35, 0.90, 0.85],
        [0.95, 0.45, 0.70],
    ];
    PALETTE[slot as usize % PALETTE.len()]
}

fn sync_job_board_from_resource(
    jobs: Option<Res<JobSystem>>,
    mut boards: Query<(&mut JobBoard, &mut crate::jobs::SmokeJobFlags)>,
) {
    let Some(jobs) = jobs else {
        return;
    };
    for (mut board, mut flags) in &mut boards {
        board.sync_from(&jobs);
        *flags = crate::jobs::SmokeJobFlags::sync_from(&jobs);
    }
}

pub fn setup_job_board(mut commands: Commands, jobs: Res<JobSystem>) {
    let mut board = JobBoard::default();
    board.sync_from(&jobs);
    let flags = crate::jobs::SmokeJobFlags::sync_from(&jobs);
    commands.spawn((
        GameplayEntity,
        board,
        flags,
        Replicated,
        Name::new("JobBoard"),
    ));
}
