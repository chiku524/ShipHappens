//! Game settings + Esc Nest menu (pause overlay).

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};
use bevy_replicon::prelude::*;

use crate::{
    account::PlayerAccount,
    boing::{self, BoingConfig, BoingStatus, ClaimVoucher},
    challenges::ChallengeBoard,
    cosmetics::{CosmeticsCatalog, EquippedCosmetic},
    flow::AppScreen,
    hub::EditorMode,
    player::{PlayerName, ThirdPersonCamera},
    season::SeasonLedger,
    session_flow::{LeaveToNestRequest, NetworkBanner},
};

#[derive(Resource, Debug, Clone)]
pub struct GameSettings {
    pub mouse_sensitivity: f32,
    pub master_volume: f32,
    pub fullscreen: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: crate::core::MOUSE_SENSITIVITY,
            master_volume: 1.0,
            fullscreen: false,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct PauseState {
    pub paused: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuPage {
    #[default]
    Main,
    Settings,
    Profile,
    Account,
    Inventory,
    Wallet,
    Market,
    Challenges,
    Controls,
}

#[derive(Component)]
struct PauseRoot;

#[derive(Component)]
struct MenuPageRoot(MenuPage);

#[derive(Component)]
struct MenuNavButton(MenuAction);

#[derive(Component)]
struct SettingsActionButton(SettingsAction);

#[derive(Component)]
struct MarketEquipButton {
    id: String,
}

#[derive(Component)]
struct MarketBoingButton(BoingAction);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MenuBodyText {
    Settings,
    Profile,
    Account,
    Inventory,
    Wallet,
    Challenges,
    MarketStatus,
}

#[derive(Component)]
struct MarketRowLabel {
    id: String,
}

#[derive(Clone, Copy, Debug)]
enum MenuAction {
    Resume,
    Open(MenuPage),
    Back,
    ReturnToNest,
    QuitGame,
}

#[derive(Clone, Copy, Debug)]
enum SettingsAction {
    VolumeDown,
    VolumeUp,
    SensDown,
    SensUp,
    ToggleFullscreen,
}

#[derive(Clone, Copy, Debug)]
enum BoingAction {
    LinkWallet,
    ClaimVoucher,
    OpenCompanion,
}

#[derive(Clone, Copy, Debug)]
enum AccountAction {
    OpenWebsite,
    LinkPendingToken,
    RefreshProfile,
    SignOut,
}

#[derive(Component)]
struct AccountActionButton(AccountAction);

const PANEL_BG: Color = Color::srgba(0.08, 0.16, 0.14, 0.94);
const BTN_BG: Color = Color::srgb(0.16, 0.30, 0.26);
const BTN_HOVER: Color = Color::srgb(0.22, 0.40, 0.34);
const BTN_PRESS: Color = Color::srgb(0.12, 0.24, 0.20);
const ACCENT: Color = Color::srgb(1.0, 0.55, 0.35);
const TEAL: Color = Color::srgb(0.35, 0.85, 0.72);
const MUTED: Color = Color::srgb(0.72, 0.82, 0.78);
const DANGER: Color = Color::srgb(0.85, 0.35, 0.32);

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .init_resource::<PauseState>()
            .init_resource::<MenuPage>()
            .add_systems(Startup, spawn_nest_menu)
            .add_systems(
                Update,
                (
                    toggle_pause.run_if(in_state(AppScreen::Playing)),
                    sync_pause_cursor.run_if(in_state(AppScreen::Playing)),
                    update_pause_visibility,
                    sync_menu_page_visibility,
                    menu_button_hover,
                    handle_menu_nav,
                    handle_settings_buttons,
                    handle_market_equip,
                    handle_market_boing,
                    handle_account_buttons,
                    sync_player_name_from_account,
                    refresh_menu_labels.run_if(in_state(AppScreen::Playing)),
                    apply_settings_hotkeys,
                ),
            );
    }
}

fn spawn_nest_menu(mut commands: Commands, catalog: Res<CosmeticsCatalog>) {
    commands
        .spawn((
            PauseRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.04, 0.82)),
            GlobalZIndex(600),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(520.0),
                    max_height: Val::Percent(90.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(22.0)),
                    row_gap: Val::Px(12.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(16.0)),
                    ..Default::default()
                },
                BackgroundColor(PANEL_BG),
                BorderColor::all(Color::srgba(1.0, 0.55, 0.35, 0.35)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("NEST MENU"),
                    TextFont {
                        font_size: FontSize::Px(28.0),
                        ..Default::default()
                    },
                    TextColor(ACCENT),
                ));
                panel.spawn((
                    Text::new("Esc closes · click a page"),
                    TextFont {
                        font_size: FontSize::Px(13.0),
                        ..Default::default()
                    },
                    TextColor(MUTED),
                ));

                spawn_page_main(panel);
                spawn_page_settings(panel);
                spawn_page_profile(panel);
                spawn_page_account(panel);
                spawn_page_inventory(panel, &catalog);
                spawn_page_wallet(panel);
                spawn_page_market(panel, &catalog);
                spawn_page_challenges(panel);
                spawn_page_controls(panel);
            });
        });
}

fn spawn_page_main(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Main),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Visible,
        ))
        .with_children(|page| {
            menu_btn(page, "Resume", MenuAction::Resume, false);
            menu_btn(page, "Settings", MenuAction::Open(MenuPage::Settings), false);
            menu_btn(page, "Profile", MenuAction::Open(MenuPage::Profile), false);
            menu_btn(page, "Account", MenuAction::Open(MenuPage::Account), false);
            menu_btn(
                page,
                "Inventory",
                MenuAction::Open(MenuPage::Inventory),
                false,
            );
            menu_btn(page, "Wallet", MenuAction::Open(MenuPage::Wallet), false);
            menu_btn(page, "Market", MenuAction::Open(MenuPage::Market), false);
            menu_btn(
                page,
                "Challenges",
                MenuAction::Open(MenuPage::Challenges),
                false,
            );
            menu_btn(page, "Controls", MenuAction::Open(MenuPage::Controls), false);
            menu_btn(page, "Return to Nest", MenuAction::ReturnToNest, false);
            menu_btn(page, "Quit Game", MenuAction::QuitGame, true);
        });
}

fn spawn_page_settings(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Settings),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Settings,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            settings_btn(page, "Volume −", SettingsAction::VolumeDown);
            settings_btn(page, "Volume +", SettingsAction::VolumeUp);
            settings_btn(page, "Sensitivity −", SettingsAction::SensDown);
            settings_btn(page, "Sensitivity +", SettingsAction::SensUp);
            settings_btn(page, "Toggle Fullscreen", SettingsAction::ToggleFullscreen);
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_profile(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Profile),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Profile"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Profile,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_account(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Account),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Account"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Account,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            account_btn(page, "Open website", AccountAction::OpenWebsite);
            account_btn(page, "Link pending token", AccountAction::LinkPendingToken);
            account_btn(page, "Refresh profile", AccountAction::RefreshProfile);
            account_btn(page, "Sign out", AccountAction::SignOut);
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_inventory(parent: &mut ChildSpawnerCommands, catalog: &CosmeticsCatalog) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Inventory),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Inventory"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Inventory,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            for item in &catalog.items {
                let id = item.id.clone();
                page.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    children![
                        (
                            MarketRowLabel { id: id.clone() },
                            Text::new(item.label.clone()),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..Default::default()
                            },
                            TextColor(MUTED),
                            Node {
                                flex_grow: 1.0,
                                ..Default::default()
                            },
                        ),
                        (
                            Button,
                            MarketEquipButton { id },
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..Default::default()
                            },
                            BackgroundColor(BTN_BG),
                            children![(
                                Text::new("Equip"),
                                TextFont {
                                    font_size: FontSize::Px(13.0),
                                    ..Default::default()
                                },
                                TextColor(Color::srgb(0.95, 0.95, 0.9)),
                            )],
                        ),
                    ],
                ));
            }
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_wallet(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Wallet),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Wallet"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Wallet,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            boing_btn(page, "Link wallet (BOING_ACCOUNT)", BoingAction::LinkWallet);
            boing_btn(page, "Prepare claim voucher", BoingAction::ClaimVoucher);
            boing_btn(page, "Open Claim Desk", BoingAction::OpenCompanion);
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_market(parent: &mut ChildSpawnerCommands, catalog: &CosmeticsCatalog) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Market),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Nest Market"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::MarketStatus,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            for item in &catalog.items {
                let id = item.id.clone();
                let label = format!("{} · {} pts", item.label, item.cost_points);
                page.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    children![
                        (
                            MarketRowLabel { id: id.clone() },
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(13.0),
                                ..Default::default()
                            },
                            TextColor(MUTED),
                            Node {
                                flex_grow: 1.0,
                                ..Default::default()
                            },
                        ),
                        (
                            Button,
                            MarketEquipButton { id },
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                justify_content: JustifyContent::Center,
                                border_radius: BorderRadius::all(Val::Px(8.0)),
                                ..Default::default()
                            },
                            BackgroundColor(BTN_BG),
                            children![(
                                Text::new("Equip"),
                                TextFont {
                                    font_size: FontSize::Px(13.0),
                                    ..Default::default()
                                },
                                TextColor(Color::srgb(0.95, 0.95, 0.9)),
                            )],
                        ),
                    ],
                ));
            }
            page.spawn((
                Text::new("Boing Network"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                TextColor(ACCENT),
            ));
            boing_btn(page, "Link wallet (BOING_ACCOUNT)", BoingAction::LinkWallet);
            boing_btn(page, "Prepare claim voucher", BoingAction::ClaimVoucher);
            boing_btn(page, "Open Claim Desk", BoingAction::OpenCompanion);
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_challenges(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Challenges),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Weekly Challenges"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                MenuBodyText::Challenges,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn spawn_page_controls(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            MenuPageRoot(MenuPage::Controls),
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                ..Default::default()
            },
            Visibility::Hidden,
        ))
        .with_children(|page| {
            page.spawn((
                Text::new("Controls"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..Default::default()
                },
                TextColor(TEAL),
            ));
            page.spawn((
                Text::new(
                    "WASD move · mouse look · scroll zoom\n\
                     Pads · E / Enter start mode\n\
                     Create Map / My Maps · C skins · M claim\n\
                     Ctrl+V link wallet · Ctrl+O claim desk\n\
                     Esc Nest menu · Q return Nest · R rematch\n\
                     ` free cursor · F11 fullscreen",
                ),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(MUTED),
            ));
            menu_btn(page, "Back", MenuAction::Back, false);
        });
}

fn menu_btn(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    action: MenuAction,
    danger: bool,
) {
    let bg = if danger { DANGER } else { BTN_BG };
    parent
        .spawn((
            Button,
            MenuNavButton(action),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(bg),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.96, 0.95, 0.9)),
            ));
        });
}

fn settings_btn(parent: &mut ChildSpawnerCommands, label: &str, action: SettingsAction) {
    parent
        .spawn((
            Button,
            SettingsActionButton(action),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(BTN_BG),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.96, 0.95, 0.9)),
            ));
        });
}

fn boing_btn(parent: &mut ChildSpawnerCommands, label: &str, action: BoingAction) {
    parent
        .spawn((
            Button,
            MarketBoingButton(action),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(BTN_BG),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.96, 0.95, 0.9)),
            ));
        });
}

fn account_btn(parent: &mut ChildSpawnerCommands, label: &str, action: AccountAction) {
    parent
        .spawn((
            Button,
            AccountActionButton(action),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..Default::default()
            },
            BackgroundColor(BTN_BG),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.96, 0.95, 0.9)),
            ));
        });
}

fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    editor: Res<EditorMode>,
    mut pause: ResMut<PauseState>,
    mut page: ResMut<MenuPage>,
    mut camera: ResMut<ThirdPersonCamera>,
) {
    if editor.active || !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    // Esc always toggles the Nest menu closed/open (no sub-page back step).
    if pause.paused {
        pause.paused = false;
        *page = MenuPage::Main;
        camera.captured = true;
    } else {
        pause.paused = true;
        *page = MenuPage::Main;
        camera.captured = false;
    }
}

fn sync_pause_cursor(
    pause: Res<PauseState>,
    camera: Res<ThirdPersonCamera>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };
    if pause.paused || !camera.captured {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

fn update_pause_visibility(
    pause: Res<PauseState>,
    mut roots: Query<&mut Visibility, With<PauseRoot>>,
) {
    let Ok(mut vis) = roots.single_mut() else {
        return;
    };
    *vis = if pause.paused {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}

fn sync_menu_page_visibility(
    pause: Res<PauseState>,
    page: Res<MenuPage>,
    mut pages: Query<(&MenuPageRoot, &mut Visibility), Without<PauseRoot>>,
) {
    for (root, mut vis) in &mut pages {
        *vis = if pause.paused && root.0 == *page {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn menu_button_hover(
    pause: Res<PauseState>,
    mut buttons: Query<
        (&Interaction, &mut BackgroundColor, Option<&MenuNavButton>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    if !pause.paused {
        return;
    }
    for (interaction, mut bg, nav) in &mut buttons {
        let danger = nav.is_some_and(|n| matches!(n.0, MenuAction::QuitGame));
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(if danger {
                    Color::srgb(0.65, 0.22, 0.2)
                } else {
                    BTN_PRESS
                });
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(if danger {
                    Color::srgb(0.95, 0.42, 0.38)
                } else {
                    BTN_HOVER
                });
            }
            Interaction::None => {
                *bg = BackgroundColor(if danger { DANGER } else { BTN_BG });
            }
        }
    }
}

fn handle_menu_nav(
    mut pause: ResMut<PauseState>,
    mut page: ResMut<MenuPage>,
    mut camera: ResMut<ThirdPersonCamera>,
    mut leave: ResMut<LeaveToNestRequest>,
    mut exit: MessageWriter<AppExit>,
    mut banner: ResMut<NetworkBanner>,
    interactions: Query<(&Interaction, &MenuNavButton), Changed<Interaction>>,
) {
    if !pause.paused {
        return;
    }
    for (interaction, nav) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match nav.0 {
            MenuAction::Resume => {
                pause.paused = false;
                camera.captured = true;
                *page = MenuPage::Main;
            }
            MenuAction::Open(next) => {
                *page = next;
            }
            MenuAction::Back => {
                *page = MenuPage::Main;
            }
            MenuAction::ReturnToNest => {
                leave.pending = true;
                pause.paused = false;
                camera.captured = true;
                *page = MenuPage::Main;
                banner.show("Returning to The Nest…", 2.5);
            }
            MenuAction::QuitGame => {
                exit.write(AppExit::Success);
            }
        }
    }
}

fn handle_settings_buttons(
    pause: Res<PauseState>,
    mut settings: ResMut<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    interactions: Query<(&Interaction, &SettingsActionButton), Changed<Interaction>>,
) {
    if !pause.paused {
        return;
    }
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        apply_settings_action(&mut settings, &mut windows, btn.0);
    }
}

fn handle_market_equip(
    pause: Res<PauseState>,
    ledger: Res<SeasonLedger>,
    catalog: Res<crate::cosmetics::CosmeticsCatalog>,
    mut equipped: ResMut<EquippedCosmetic>,
    mut banner: ResMut<NetworkBanner>,
    mut commands: Commands,
    mut colors: Query<&mut crate::player::PlayerColor, With<crate::player::LocalPlayer>>,
    client: Option<Res<bevy_replicon_renet::RenetClient>>,
    interactions: Query<(&Interaction, &MarketEquipButton), Changed<Interaction>>,
) {
    if !pause.paused {
        return;
    }
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if ledger.unlocked.contains(&btn.id) {
            equipped.id = btn.id.clone();
            let tint = catalog
                .items
                .iter()
                .find(|i| i.id == btn.id)
                .map(|i| i.tint)
                .unwrap_or([0.95, 0.45, 0.35]);
            if client.is_some() {
                commands.client_trigger(crate::cosmetics::EquipCosmeticRequest {
                    id: btn.id.clone(),
                    tint,
                });
            } else if let Ok(mut color) = colors.single_mut() {
                color.0 = tint;
            }
            banner.show(format!("Equipped {}", btn.id), 2.0);
        } else {
            banner.show("Skin locked — earn more season points", 2.5);
        }
    }
}

fn handle_account_buttons(
    pause: Res<PauseState>,
    mut account: ResMut<PlayerAccount>,
    mut banner: ResMut<NetworkBanner>,
    interactions: Query<(&Interaction, &AccountActionButton), Changed<Interaction>>,
) {
    if !pause.paused {
        return;
    }
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let note = match btn.0 {
            AccountAction::OpenWebsite => crate::account::open_website(&mut account),
            AccountAction::LinkPendingToken => crate::account::link_pending_token(&mut account),
            AccountAction::RefreshProfile => crate::account::refresh_profile(&mut account),
            AccountAction::SignOut => {
                account.clear();
                account.note.clone()
            }
        };
        banner.show(note, 3.5);
    }
}

fn sync_player_name_from_account(
    account: Res<PlayerAccount>,
    mut names: Query<&mut PlayerName, With<crate::player::LocalPlayer>>,
) {
    if !account.is_changed() || !account.signed_in() {
        return;
    }
    if let Ok(mut name) = names.single_mut() {
        name.0 = account.display_name.clone();
    }
}

fn handle_market_boing(
    pause: Res<PauseState>,
    mut config: ResMut<BoingConfig>,
    ledger: Res<SeasonLedger>,
    equipped: Res<EquippedCosmetic>,
    mut voucher: ResMut<ClaimVoucher>,
    mut banner: ResMut<NetworkBanner>,
    interactions: Query<(&Interaction, &MarketBoingButton), Changed<Interaction>>,
) {
    if !pause.paused {
        return;
    }
    for (interaction, btn) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match btn.0 {
            BoingAction::LinkWallet => {
                let note = boing::link_wallet_from_env(&mut config, &mut voucher);
                banner.show(note, 3.0);
            }
            BoingAction::ClaimVoucher => {
                let note = boing::prepare_claim_voucher(
                    &config,
                    &ledger,
                    &equipped,
                    &mut voucher,
                );
                banner.show(note, 3.5);
            }
            BoingAction::OpenCompanion => {
                let note = boing::open_claim_companion_page(&mut voucher);
                banner.show(note, 3.0);
            }
        }
    }
}

fn refresh_menu_labels(
    pause: Res<PauseState>,
    page: Res<MenuPage>,
    settings: Res<GameSettings>,
    ledger: Res<SeasonLedger>,
    equipped: Res<EquippedCosmetic>,
    catalog: Res<CosmeticsCatalog>,
    config: Res<BoingConfig>,
    status: Res<BoingStatus>,
    board: Res<ChallengeBoard>,
    voucher: Res<ClaimVoucher>,
    account: Res<PlayerAccount>,
    mut bodies: Query<(&MenuBodyText, &mut Text), Without<MarketRowLabel>>,
    mut market_rows: Query<(&MarketRowLabel, &mut Text), Without<MenuBodyText>>,
) {
    if !pause.paused {
        return;
    }

    for (kind, mut text) in &mut bodies {
        match (*kind, *page) {
            (MenuBodyText::Settings, MenuPage::Settings) => {
                **text = format!(
                    "Volume {:.0}% · Sensitivity {:.4} · Fullscreen {}",
                    settings.master_volume * 100.0,
                    settings.mouse_sensitivity,
                    if settings.fullscreen { "ON" } else { "off" }
                );
            }
            (MenuBodyText::Profile, MenuPage::Profile) => {
                let wallet = truncate_wallet(
                    account
                        .boing_wallet
                        .as_deref()
                        .or(config.linked_account.as_deref()),
                );
                let skin = catalog
                    .items
                    .iter()
                    .find(|i| i.id == equipped.id)
                    .map(|i| i.label.as_str())
                    .unwrap_or(equipped.id.as_str());
                let who = if account.signed_in() {
                    format!("{} ({})", account.display_name, account.email)
                } else {
                    "Guest — open Account to sign in".into()
                };
                **text = format!(
                    "{who}\nSeason {} · {} pts · {} parties\nSkin: {}\nWallet: {}\nInventory / Wallet / Account for more",
                    ledger.season_id,
                    ledger.points,
                    ledger.parties_played,
                    skin,
                    wallet,
                );
            }
            (MenuBodyText::Account, MenuPage::Account) => {
                if account.signed_in() {
                    let wallet = account
                        .boing_wallet
                        .as_deref()
                        .unwrap_or("(no Boing wallet)");
                    **text = format!(
                        "Signed in as {}\n{}\nBoing wallet {}\nAPI {}\n{}\nDrop pending_token.txt into {}",
                        account.display_name,
                        account.email,
                        wallet,
                        account.api_base,
                        if account.note.is_empty() {
                            "Ready."
                        } else {
                            account.note.as_str()
                        },
                        PlayerAccount::pending_token_path().display()
                    );
                } else {
                    **text = format!(
                        "Not signed in.\nRestart the game to use the Sign In / Register intro,\nor link a website token at {}\nAPI: {}\n{}",
                        PlayerAccount::pending_token_path().display(),
                        account.api_base,
                        if account.note.is_empty() {
                            ""
                        } else {
                            account.note.as_str()
                        }
                    );
                }
            }
            (MenuBodyText::Inventory, MenuPage::Inventory) => {
                let owned = ledger.unlocked.len();
                let total = catalog.items.len();
                let skin = catalog
                    .items
                    .iter()
                    .find(|i| i.id == equipped.id)
                    .map(|i| i.label.as_str())
                    .unwrap_or(equipped.id.as_str());
                **text = format!(
                    "Owned {owned}/{total} cosmetics · equipped: {skin}\nUnlock more in Market with season points."
                );
            }
            (MenuBodyText::Wallet, MenuPage::Wallet) => {
                let cloud = truncate_wallet(account.boing_wallet.as_deref());
                let local = truncate_wallet(config.linked_account.as_deref());
                let balance = status
                    .native_balance
                    .clone()
                    .unwrap_or_else(|| "—".into());
                let reach = if status.reachable {
                    "reachable"
                } else if status.last_error.is_empty() {
                    "unknown"
                } else {
                    "unreachable"
                };
                **text = format!(
                    "Season soft balance: {} pts (season {})\n\
                     Cloud wallet: {}\n\
                     Session wallet: {}\n\
                     Boing chain {} · RPC {}\n\
                     On-chain balance: {}\n\
                     Claim note: {}",
                    ledger.points,
                    ledger.season_id,
                    cloud,
                    local,
                    config.chain_id,
                    reach,
                    balance,
                    if voucher.note.is_empty() {
                        "none yet"
                    } else {
                        voucher.note.as_str()
                    }
                );
            }
            (MenuBodyText::Challenges, MenuPage::Challenges) => {
                if board.defs.is_empty() {
                    **text = "No weekly challenges loaded.".into();
                } else {
                    let mut lines = vec![format!("Week {}", board.week)];
                    for d in &board.defs {
                        let p = board.progress.get(&d.id).copied().unwrap_or(0);
                        let mark = if board.claimed.contains(&d.id) {
                            "[done]"
                        } else if p >= d.target {
                            "[ready]"
                        } else {
                            "[…]"
                        };
                        lines.push(format!(
                            "{} {} — {}/{} (+{} pts)",
                            mark,
                            d.label,
                            p.min(d.target),
                            d.target,
                            d.reward_points
                        ));
                    }
                    **text = lines.join("\n");
                }
            }
            (MenuBodyText::MarketStatus, MenuPage::Market) => {
                **text = format!(
                    "{} season pts · equipped {} · {}",
                    ledger.points,
                    equipped.id,
                    if voucher.note.is_empty() {
                        "Boing ready"
                    } else {
                        voucher.note.as_str()
                    }
                );
            }
            _ => {}
        }
    }

    if matches!(*page, MenuPage::Market | MenuPage::Inventory) {
        for (row, mut text) in &mut market_rows {
            let Some(item) = catalog.items.iter().find(|i| i.id == row.id) else {
                continue;
            };
            if matches!(*page, MenuPage::Inventory) {
                let state = if equipped.id == row.id {
                    "Equipped"
                } else if ledger.unlocked.contains(&row.id) {
                    "Owned — Equip"
                } else {
                    "Locked"
                };
                **text = format!("{} · {state}", item.label);
            } else {
                let state = if equipped.id == row.id {
                    "Equipped"
                } else if ledger.unlocked.contains(&row.id) {
                    "Owned"
                } else {
                    "Locked"
                };
                **text = format!("{} · {} pts · {state}", item.label, item.cost_points);
            }
        }
    }
}

fn truncate_wallet(account: Option<&str>) -> String {
    match account {
        Some(a) if a.len() > 14 => format!("{}…{}", &a[..8], &a[a.len() - 4..]),
        Some(a) => a.to_string(),
        None => "not linked".into(),
    }
}

fn apply_settings_hotkeys(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GameSettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        apply_settings_action(&mut settings, &mut windows, SettingsAction::VolumeDown);
    }
    if keyboard.just_pressed(KeyCode::BracketRight) {
        apply_settings_action(&mut settings, &mut windows, SettingsAction::VolumeUp);
    }
    if keyboard.just_pressed(KeyCode::Minus) {
        apply_settings_action(&mut settings, &mut windows, SettingsAction::SensDown);
    }
    if keyboard.just_pressed(KeyCode::Equal) {
        apply_settings_action(&mut settings, &mut windows, SettingsAction::SensUp);
    }
    if keyboard.just_pressed(KeyCode::F11) {
        apply_settings_action(
            &mut settings,
            &mut windows,
            SettingsAction::ToggleFullscreen,
        );
    }
}

fn apply_settings_action(
    settings: &mut GameSettings,
    windows: &mut Query<&mut Window, With<PrimaryWindow>>,
    action: SettingsAction,
) {
    match action {
        SettingsAction::VolumeDown => {
            settings.master_volume = (settings.master_volume - 0.1).clamp(0.0, 1.0);
        }
        SettingsAction::VolumeUp => {
            settings.master_volume = (settings.master_volume + 0.1).clamp(0.0, 1.0);
        }
        SettingsAction::SensDown => {
            settings.mouse_sensitivity =
                (settings.mouse_sensitivity - 0.0004).clamp(0.0005, 0.008);
        }
        SettingsAction::SensUp => {
            settings.mouse_sensitivity =
                (settings.mouse_sensitivity + 0.0004).clamp(0.0005, 0.008);
        }
        SettingsAction::ToggleFullscreen => {
            settings.fullscreen = !settings.fullscreen;
            if let Ok(mut window) = windows.single_mut() {
                window.mode = if settings.fullscreen {
                    WindowMode::BorderlessFullscreen(MonitorSelection::Current)
                } else {
                    WindowMode::Windowed
                };
            }
        }
    }
}
