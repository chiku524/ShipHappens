//! Legacy desk overlay helpers (plaza is the primary entry now).
#![allow(dead_code)]

use bevy::prelude::*;

use crate::{
    core::DEFAULT_PORT,
    flow::AppScreen,
    network::SessionBooted,
    ui::menu_options::MenuPartyOptions,
    Cli,
};

#[derive(Component)]
pub struct TitleRoot;

#[derive(Component)]
pub(super) struct MenuHintText;

#[derive(Component)]
pub(super) struct MenuBodyText;

#[derive(Component)]
pub(super) struct MenuOptionsText;

pub fn spawn_title_screen(mut commands: Commands) {
    spawn_title_ui(&mut commands);
}

pub fn spawn_title_ui(commands: &mut Commands) {
    commands.spawn((
        TitleRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(12.0),
            ..Default::default()
        },
        BackgroundColor(Color::srgba(0.06, 0.04, 0.12, 0.94)),
        GlobalZIndex(500),
        children![
            (
                Text::new("PUGDYMON"),
                TextFont {
                    font_size: FontSize::Px(60.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 0.55, 0.35)),
            ),
            (
                Text::new("Party Saga"),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.35, 0.85, 0.75)),
            ),
            (
                Text::new("Pugdy Monsters · The Nest · Race · Vibe · Shooter · Boing rewards"),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.75, 0.78, 0.88)),
            ),
            (
                MenuBodyText,
                Text::new("1  Solo party\n2  Host LAN\n3  Join LAN"),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..Default::default()
                },
            ),
            (
                MenuOptionsText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.65, 0.85, 0.75)),
            ),
            (
                MenuHintText,
                Text::new("Space solo · J join IP · Enter ready in hub · C skins · M claim"),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.55, 0.6, 0.7)),
            ),
        ],
    ));
}

pub(super) fn sync_title_banner(
    banner: Res<crate::session_flow::NetworkBanner>,
    options: Res<MenuPartyOptions>,
    season: Res<crate::season::SeasonLedger>,
    boing: Res<crate::boing::BoingConfig>,
    mut hints: Query<&mut Text, (With<MenuHintText>, Without<MenuOptionsText>)>,
    mut opts_text: Query<&mut Text, (With<MenuOptionsText>, Without<MenuHintText>)>,
    mut body: Query<&mut Text, (With<MenuBodyText>, Without<MenuHintText>, Without<MenuOptionsText>)>,
) {
    if let Ok(mut text) = opts_text.single_mut() {
        let wallet = boing
            .linked_account
            .as_ref()
            .map(|a| format!("{}…", &a[..10.min(a.len())]))
            .unwrap_or_else(|| "no wallet".into());
        **text = format!(
            "Season {} pts · J join {} · wallet {wallet}",
            season.points,
            options.join_host_label()
        );
    }
    if let Ok(mut text) = body.single_mut() {
        **text = format!(
            "1  Solo party (bots)\n2  Host LAN{}\n3  Join LAN ({})",
            if options.dedicated_host {
                " (dedicated)"
            } else {
                ""
            },
            options.join_host_label(),
        );
    }
    let Ok(mut text) = hints.single_mut() else {
        return;
    };
    if banner.message.is_empty() {
        **text =
            "Space solo · J join IP · B bracket · D dedicated · Ctrl+V link BOING_ACCOUNT".into();
    } else {
        **text = banner.message.clone();
    }
}

pub(super) fn tune_menu_options(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut options: ResMut<MenuPartyOptions>,
) {
    if keyboard.just_pressed(KeyCode::KeyB) {
        options.cycle_bracket();
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        options.dedicated_host = !options.dedicated_host;
    }
    if keyboard.just_pressed(KeyCode::KeyJ) {
        options.cycle_join_preset();
    }
}

pub fn start_from_title(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<AppScreen>>,
    mut cli: ResMut<Cli>,
    mut booted: ResMut<SessionBooted>,
    options: Res<MenuPartyOptions>,
    mut director: ResMut<crate::party::PartyDirector>,
    mut ready: ResMut<crate::party::HubReady>,
    roots: Query<Entity, With<TitleRoot>>,
    mut commands: Commands,
    channels: Res<bevy_replicon::prelude::RepliconChannels>,
    mut registry: ResMut<crate::player::PlayerRegistry>,
    mut slots: ResMut<crate::network::PlayerSlotCounter>,
    defaults: Res<crate::data::PlayerDefaults>,
    spawn: Res<crate::party::PartySpawn>,
) {
    let choice = if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Space)
    {
        Some(MenuChoice::Local)
    } else if keyboard.just_pressed(KeyCode::Digit2) {
        Some(MenuChoice::Host)
    } else if keyboard.just_pressed(KeyCode::Digit3) {
        Some(MenuChoice::Join)
    } else {
        None
    };

    let Some(choice) = choice else {
        return;
    };

    match choice {
        MenuChoice::Local => *cli = Cli::Local,
        MenuChoice::Host => {
            *cli = Cli::Host {
                port: DEFAULT_PORT,
                bracket_size: options.bracket_size.clamp(2, 16),
                production_timers: options.production_timers,
                dedicated: options.dedicated_host,
            };
        }
        MenuChoice::Join => {
            *cli = Cli::Join {
                address: options.join_address(),
                port: options.join_port,
            };
        }
    }

    director.reset_party();
    ready.host_ready = false;

    // Adapt RoomSpawnPoint-less boot: temporarily insert spawn via party spawn.
    let spawn_point = crate::rooms::RoomSpawnPoint {
        lobby: spawn.hub,
        current: spawn.hub,
    };
    commands.insert_resource(spawn_point);

    if let Err(err) = crate::network::boot_session(
        &mut commands,
        &mut booted,
        cli.as_ref(),
        channels.as_ref(),
        &mut registry,
        &mut slots,
        Some(&crate::rooms::RoomSpawnPoint {
            lobby: spawn.hub,
            current: spawn.hub,
        }),
        defaults.as_ref(),
    ) {
        warn!("session boot failed: {err}");
        return;
    }

    for entity in &roots {
        commands.entity(entity).despawn();
    }
    next.set(AppScreen::Playing);
}

enum MenuChoice {
    Local,
    Host,
    Join,
}

pub fn skip_title_if_needed(
    cli: Res<Cli>,
    mut next: ResMut<NextState<AppScreen>>,
    mut booted: ResMut<SessionBooted>,
    roots: Query<Entity, With<TitleRoot>>,
    mut commands: Commands,
    channels: Res<bevy_replicon::prelude::RepliconChannels>,
    mut registry: ResMut<crate::player::PlayerRegistry>,
    mut slots: ResMut<crate::network::PlayerSlotCounter>,
    defaults: Res<crate::data::PlayerDefaults>,
    spawn: Res<crate::party::PartySpawn>,
) {
    if !crate::flow::should_skip_title(cli.as_ref()) {
        return;
    }
    let spawn_point = crate::rooms::RoomSpawnPoint {
        lobby: spawn.hub,
        current: spawn.hub,
    };
    commands.insert_resource(spawn_point.clone());
    if let Err(err) = crate::network::boot_session(
        &mut commands,
        &mut booted,
        cli.as_ref(),
        channels.as_ref(),
        &mut registry,
        &mut slots,
        Some(&spawn_point),
        defaults.as_ref(),
    ) {
        warn!("session boot failed: {err}");
    }
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    next.set(AppScreen::Playing);
}
