//! Intro Sign In / Register — required before Nest boot for interactive local play.

use bevy::{
    input::{
        keyboard::{Key, KeyboardInput},
        ButtonState,
    },
    prelude::*,
};

use crate::{
    account::{login, signup, PlayerAccount},
    flow::AppScreen,
    network::SessionBooted,
};

#[derive(Component)]
pub struct AuthRoot;

#[derive(Component)]
struct AuthBodyText;

#[derive(Component)]
struct AuthStatusText;

#[derive(Component)]
struct AuthHintText;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum AuthUiButton {
    ModeSignIn,
    ModeRegister,
    Submit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthMode {
    #[default]
    SignIn,
    Register,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AuthField {
    #[default]
    Email,
    Password,
    DisplayName,
}

#[derive(Resource, Debug)]
pub struct AuthForm {
    pub mode: AuthMode,
    focus: AuthField,
    email: String,
    password: String,
    display_name: String,
    status: String,
    busy: bool,
}

impl Default for AuthForm {
    fn default() -> Self {
        Self {
            mode: AuthMode::SignIn,
            focus: AuthField::Email,
            email: String::new(),
            password: String::new(),
            display_name: String::new(),
            status: String::new(),
            busy: false,
        }
    }
}

impl AuthForm {
    fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            AuthField::Email => &mut self.email,
            AuthField::Password => &mut self.password,
            AuthField::DisplayName => &mut self.display_name,
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.mode {
            AuthMode::SignIn => match self.focus {
                AuthField::Email => AuthField::Password,
                _ => AuthField::Email,
            },
            AuthMode::Register => match self.focus {
                AuthField::DisplayName => AuthField::Email,
                AuthField::Email => AuthField::Password,
                AuthField::Password => AuthField::DisplayName,
            },
        };
    }

    fn set_mode(&mut self, mode: AuthMode) {
        self.mode = mode;
        self.focus = match mode {
            AuthMode::SignIn => AuthField::Email,
            AuthMode::Register => AuthField::DisplayName,
        };
        self.status.clear();
    }
}

pub struct AuthIntroPlugin;

impl Plugin for AuthIntroPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AuthForm>().add_systems(
            Startup,
            spawn_auth_intro.run_if(in_state(AppScreen::Title)),
        ).add_systems(
            Update,
            (
                enter_play_if_signed_in,
                handle_auth_buttons,
                handle_auth_typing,
                sync_auth_ui,
            )
                .chain()
                .run_if(in_state(AppScreen::Title)),
        );
    }
}

fn spawn_auth_intro(mut commands: Commands, account: Res<PlayerAccount>) {
    let api = account.api_base.clone();
    commands.spawn((
        AuthRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(10.0),
            ..Default::default()
        },
        BackgroundColor(Color::srgba(0.05, 0.08, 0.1, 0.96)),
        GlobalZIndex(500),
        children![
            (
                Text::new("PUDGYMON"),
                TextFont {
                    font_size: FontSize::Px(58.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(1.0, 0.55, 0.35)),
            ),
            (
                Text::new("Party Saga"),
                TextFont {
                    font_size: FontSize::Px(28.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.35, 0.85, 0.75)),
            ),
            (
                Text::new("Sign in to enter The Nest"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.75, 0.78, 0.88)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..Default::default()
                },
            ),
            (
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..Default::default()
                },
                children![
                    auth_mode_button("Sign In", AuthUiButton::ModeSignIn),
                    auth_mode_button("Register", AuthUiButton::ModeRegister),
                ],
            ),
            (
                AuthBodyText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.95, 0.9)),
                Node {
                    margin: UiRect::top(Val::Px(12.0)),
                    ..Default::default()
                },
            ),
            auth_action_button("Continue", AuthUiButton::Submit),
            (
                AuthStatusText,
                Text::new(format!("API {api}")),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.7, 0.8, 0.75)),
                Node {
                    margin: UiRect::top(Val::Px(8.0)),
                    max_width: Val::Px(520.0),
                    ..Default::default()
                },
            ),
            (
                AuthHintText,
                Text::new("Tab field · F1 Sign In · F2 Register · Enter submit · type to edit"),
                TextFont {
                    font_size: FontSize::Px(12.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.55, 0.6, 0.7)),
            ),
        ],
    ));
}

fn auth_mode_button(label: &str, action: AuthUiButton) -> impl Bundle {
    (
        Button,
        action,
        Node {
            padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.12, 0.18, 0.2)),
        BorderColor::all(Color::srgb(0.35, 0.55, 0.5)),
        children![(
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(16.0),
                ..Default::default()
            },
            TextColor(Color::srgb(0.9, 0.95, 0.92)),
        )],
    )
}

fn auth_action_button(label: &str, action: AuthUiButton) -> impl Bundle {
    (
        Button,
        action,
        Node {
            margin: UiRect::top(Val::Px(10.0)),
            padding: UiRect::axes(Val::Px(28.0), Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..Default::default()
        },
        BackgroundColor(Color::srgb(0.85, 0.4, 0.28)),
        BorderColor::all(Color::srgb(1.0, 0.55, 0.35)),
        children![(
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..Default::default()
            },
            TextColor(Color::srgb(0.98, 0.96, 0.92)),
        )],
    )
}

fn enter_play_if_signed_in(
    account: Res<PlayerAccount>,
    mut next: ResMut<NextState<AppScreen>>,
    mut booted: ResMut<SessionBooted>,
    roots: Query<Entity, With<AuthRoot>>,
    mut commands: Commands,
    channels: Res<bevy_replicon::prelude::RepliconChannels>,
    mut registry: ResMut<crate::player::PlayerRegistry>,
    mut slots: ResMut<crate::network::PlayerSlotCounter>,
    defaults: Res<crate::data::PlayerDefaults>,
    spawn: Res<crate::party::PartySpawn>,
    cli: Res<crate::Cli>,
) {
    if !account.signed_in() {
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
        warn!("session boot failed after auth: {err}");
        return;
    }

    for entity in &roots {
        commands.entity(entity).despawn();
    }
    next.set(AppScreen::Playing);
}

fn handle_auth_buttons(
    mut form: ResMut<AuthForm>,
    mut account: ResMut<PlayerAccount>,
    interactions: Query<(&Interaction, &AuthUiButton), Changed<Interaction>>,
) {
    for (interaction, button) in &interactions {
        if *interaction != Interaction::Pressed || form.busy {
            continue;
        }
        match button {
            AuthUiButton::ModeSignIn => form.set_mode(AuthMode::SignIn),
            AuthUiButton::ModeRegister => form.set_mode(AuthMode::Register),
            AuthUiButton::Submit => submit_auth(&mut form, &mut account),
        }
    }
}

fn handle_auth_typing(
    mut form: ResMut<AuthForm>,
    mut account: ResMut<PlayerAccount>,
    mut reader: MessageReader<KeyboardInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if form.busy {
        return;
    }

    if keyboard.just_pressed(KeyCode::F1) {
        form.set_mode(AuthMode::SignIn);
    }
    if keyboard.just_pressed(KeyCode::F2) {
        form.set_mode(AuthMode::Register);
    }

    for event in reader.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Tab => {
                form.cycle_focus();
                continue;
            }
            Key::Enter => {
                submit_auth(&mut form, &mut account);
                continue;
            }
            Key::Backspace => {
                form.focused_mut().pop();
                continue;
            }
            _ => {}
        }
        if let Some(text) = &event.text {
            for ch in text.chars() {
                if ch.is_control() {
                    continue;
                }
                let field = form.focused_mut();
                if field.len() < 96 {
                    field.push(ch);
                }
            }
        }
    }
}

fn submit_auth(form: &mut AuthForm, account: &mut PlayerAccount) {
    if form.busy {
        return;
    }
    let email = form.email.trim().to_string();
    let password = form.password.clone();
    if email.is_empty() || password.is_empty() {
        form.status = "Email and password are required.".into();
        return;
    }
    if password.len() < 8 {
        form.status = "Password must be at least 8 characters.".into();
        return;
    }

    form.busy = true;
    form.status = "Contacting accounts…".into();

    let result = match form.mode {
        AuthMode::SignIn => login(account, &email, &password),
        AuthMode::Register => {
            let name = form.display_name.trim().to_string();
            if name.is_empty() {
                form.busy = false;
                form.status = "Display name is required to register.".into();
                return;
            }
            signup(account, &email, &password, &name)
        }
    };

    form.busy = false;
    form.status = result;
    if account.signed_in() {
        form.password.clear();
    }
}

fn sync_auth_ui(
    form: Res<AuthForm>,
    account: Res<PlayerAccount>,
    mut body: Query<&mut Text, (With<AuthBodyText>, Without<AuthStatusText>, Without<AuthHintText>)>,
    mut status: Query<&mut Text, (With<AuthStatusText>, Without<AuthBodyText>, Without<AuthHintText>)>,
) {
    let mark = |field: AuthField| -> &'static str {
        if form.focus == field {
            ">"
        } else {
            " "
        }
    };
    let masked: String = form.password.chars().map(|_| '*').collect();

    if let Ok(mut text) = body.single_mut() {
        **text = match form.mode {
            AuthMode::SignIn => format!(
                "Mode: Sign In\n{} Email: {}\n{} Password: {}",
                mark(AuthField::Email),
                if form.email.is_empty() {
                    "_"
                } else {
                    form.email.as_str()
                },
                mark(AuthField::Password),
                if masked.is_empty() { "_" } else { masked.as_str() },
            ),
            AuthMode::Register => format!(
                "Mode: Register\n{} Name: {}\n{} Email: {}\n{} Password: {}",
                mark(AuthField::DisplayName),
                if form.display_name.is_empty() {
                    "_"
                } else {
                    form.display_name.as_str()
                },
                mark(AuthField::Email),
                if form.email.is_empty() {
                    "_"
                } else {
                    form.email.as_str()
                },
                mark(AuthField::Password),
                if masked.is_empty() { "_" } else { masked.as_str() },
            ),
        };
    }

    if let Ok(mut text) = status.single_mut() {
        if form.status.is_empty() {
            **text = format!("API {}", account.api_base);
        } else {
            **text = form.status.clone();
        }
    }
}
