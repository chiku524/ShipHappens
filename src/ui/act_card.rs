//! Brief act cards when a vault room starts — tournament story beats.

use bevy::prelude::*;

use crate::{
    flow::AppScreen,
    tournament::{TournamentDirector, TournamentPhase, TournamentSnapshot},
};

#[derive(Resource, Debug, Default)]
pub struct ActCard {
    pub timer: f32,
    pub title: String,
    pub body: String,
    last_phase: Option<TournamentPhase>,
    last_room_index: Option<usize>,
}

#[derive(Component)]
pub(super) struct ActCardRoot;

#[derive(Component)]
pub(super) struct ActCardTitle;

#[derive(Component)]
pub(super) struct ActCardBody;

pub(super) fn spawn_act_card(mut commands: Commands) {
    commands.spawn((
        ActCardRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(14.0),
            ..Default::default()
        },
        BackgroundColor(Color::srgba(0.02, 0.05, 0.1, 0.88)),
        GlobalZIndex(450),
        Visibility::Hidden,
        children![
            (
                ActCardTitle,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(44.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.95, 0.78, 0.28)),
            ),
            (
                ActCardBody,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..Default::default()
                },
                TextColor(Color::srgb(0.85, 0.9, 0.95)),
            ),
        ],
    ));
}

pub(super) fn sync_act_card(
    director: Res<TournamentDirector>,
    snapshots: Query<&TournamentSnapshot>,
    mut card: ResMut<ActCard>,
) {
    let snap = snapshots.iter().next();
    let phase = snap.map(|s| s.phase).unwrap_or(director.phase);
    let room = snap.map(|s| s.room).unwrap_or(director.room);
    let room_index = director.room_index;

    let phase_changed = card.last_phase != Some(phase);
    let room_changed = card.last_room_index != Some(room_index);
    card.last_phase = Some(phase);
    card.last_room_index = Some(room_index);

    if !phase_changed && !room_changed {
        return;
    }

    match phase {
        TournamentPhase::RoomActive => {
            card.title = format!("ACT {} — {}", room_index + 1, room.label());
            card.body = act_blurb(room_index, false);
            card.timer = 2.6;
        }
        TournamentPhase::Finale => {
            card.title = "FINALE — Shuttle Bay Meltdown".into();
            card.body = act_blurb(3, true);
            card.timer = 2.8;
        }
        TournamentPhase::Elimination => {
            card.title = "VOLUNTARY SEPARATION".into();
            card.body = "Bottom of the bracket reports to the airlock.\nSurvivors — stretch those contractor legs.".into();
            card.timer = 2.2;
        }
        TournamentPhase::Podium => {
            card.title = "CORPORATE PODIUM".into();
            card.body = "Smile for the shareholders.\nR rematch · Q menu".into();
            card.timer = 2.4;
        }
        _ => {}
    }
}

fn act_blurb(room_index: usize, finale: bool) -> String {
    if finale {
        return "Coolant · load · seal.\nMeltdown does not wait for paperwork.".into();
    }
    match room_index {
        0 => "Sort freight. Labels are optional. Compliance is not.".into(),
        1 => "Pick up crates. Deliver to the green pad.\nDropping is a write-up.".into(),
        2 => "Panel labels lie.\nLeaseholder knows the order — everyone else pings (V).".into(),
        _ => "Heroism is voluntary.".into(),
    }
}

pub(super) fn tick_act_card(
    time: Res<Time>,
    mut card: ResMut<ActCard>,
    mut root: Query<&mut Visibility, With<ActCardRoot>>,
    mut title: Query<&mut Text, (With<ActCardTitle>, Without<ActCardBody>)>,
    mut body: Query<&mut Text, (With<ActCardBody>, Without<ActCardTitle>)>,
) {
    if card.timer > 0.0 {
        card.timer -= time.delta_secs();
    }

    let visible = card.timer > 0.0;
    if let Ok(mut vis) = root.single_mut() {
        *vis = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if visible {
        if let Ok(mut t) = title.single_mut() {
            **t = card.title.clone();
        }
        if let Ok(mut b) = body.single_mut() {
            **b = card.body.clone();
        }
    }
}

pub(super) fn act_card_systems_active(screen: Res<State<AppScreen>>) -> bool {
    *screen.get() == AppScreen::Playing
}
