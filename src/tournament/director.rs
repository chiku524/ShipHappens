use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{RenetClient, RenetServer};

use super::authority::is_tournament_authority;
use super::bracket::{
    TournamentConfig, TournamentDirector, TournamentSnapshot, DEFAULT_ONLINE_BRACKET_SIZE,
    ELIMINATION_DURATION_SECS,
};
use super::types::{RoomId, SlotId, TournamentPhase};
use crate::{
    announcer::AnnouncerQueue,
    audio_fx::VoKind,
    core::ROOM_CLEAR_GRACE_SECS,
    economy::{PracticeLedger, PayoutCalculator},
    flow::AppScreen,
    juice::{play_juice, CameraShake, FeedbackFlash, JuiceEvent},
    player::{NetworkPlayer, PlayerName},
    rooms::RoomRuntime,
    scoring::{PlayerScoreId, ScoringService},
    Cli,
};

pub struct TournamentPlugin;

impl Plugin for TournamentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TournamentConfig>()
            .init_resource::<TournamentDirector>()
            .init_resource::<super::lobby::LobbyGate>()
            .replicate::<TournamentSnapshot>()
            .add_systems(
                Startup,
                (
                    configure_tournament,
                    super::lobby::configure_lobby_gate,
                    init_tournament,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    super::lobby::lobby_ready_input,
                    super::lobby::apply_lobby_gate,
                    claim_bracket_slot_for_player,
                    tick_tournament_director,
                    bot_room_progress,
                    sync_tournament_snapshot,
                )
                    .chain()
                    .run_if(is_tournament_authority)
                    .run_if(in_state(AppScreen::Playing)),
            );
    }
}

pub fn configure_tournament(mut config: ResMut<TournamentConfig>, cli: Res<Cli>) {
    match cli.as_ref() {
        Cli::Local => {
            config.bracket_size = super::bracket::DEFAULT_DEV_BRACKET_SIZE;
            config.fast_timers = true;
        }
        Cli::Host {
            bracket_size,
            production_timers,
            ..
        } => {
            config.bracket_size = (*bracket_size).clamp(2, DEFAULT_ONLINE_BRACKET_SIZE);
            config.fast_timers = !production_timers;
        }
        Cli::Join { .. } => {}
    }
}

fn init_tournament(
    mut commands: Commands,
    cli: Res<Cli>,
    config: Res<TournamentConfig>,
    mut director: ResMut<TournamentDirector>,
    mut scoring: ResMut<ScoringService>,
    mut ledger: ResMut<PracticeLedger>,
    server: Option<Res<RenetServer>>,
    client: Option<Res<RenetClient>>,
) {
    if !is_tournament_authority(server, client) {
        return;
    }

    *director = TournamentDirector::bootstrap(&config);
    scoring.reset_room();
    for slot in &director.slots {
        let players = (0..slot.size.player_count())
            .map(|p| PlayerScoreId(slot.id.0 * 10 + p as u32))
            .collect();
        scoring.register_slot(slot.id.clone(), players);
    }
    ledger.reset_for_tournament(config.match_mode, director.slots.len());
    commands.spawn((
        TournamentSnapshot::default(),
        Replicated,
        Name::new("TournamentSnapshot"),
    ));
    info!(
        "tournament bootstrapped: {} {:?} slots, mode {:?}, fast_timers={}",
        director.slots.len(),
        config.slot_size,
        config.match_mode,
        config.fast_timers,
    );
    let _ = cli;
}

fn claim_bracket_slot_for_player(
    config: Res<TournamentConfig>,
    mut director: ResMut<TournamentDirector>,
    players: Query<(&NetworkPlayer, &PlayerName), Added<NetworkPlayer>>,
) {
    if director.phase != TournamentPhase::Lobby {
        return;
    }

    for (network_player, name) in &players {
        let team = crate::tournament::types::bracket_team_index(network_player.slot, config.slot_size)
            as usize;
        let Some(slot) = director.slots.get_mut(team) else {
            continue;
        };
        slot.is_bot = false;
        if slot.display_name.starts_with("Bot") || slot.display_name.is_empty() {
            slot.display_name = name.0.clone();
        } else if !slot.display_name.contains(&name.0) {
            slot.display_name = format!("{}, {}", slot.display_name, name.0);
        }
    }
}

fn tick_tournament_director(
    time: Res<Time>,
    config: Res<TournamentConfig>,
    mut director: ResMut<TournamentDirector>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    mut announcer: ResMut<AnnouncerQueue>,
    mut ledger: ResMut<PracticeLedger>,
    mut mastery: ResMut<crate::meta::RoomMastery>,
    mut vo: ResMut<crate::audio_fx::VoQueue>,
    mut shake: ResMut<CameraShake>,
    mut flash: ResMut<FeedbackFlash>,
    mut audio: ResMut<crate::audio_fx::AudioFxQueue>,
) {
    // Early advance when vault objectives clear (don't wait out the full timer).
    if matches!(
        director.phase,
        TournamentPhase::RoomActive | TournamentPhase::Finale
    ) && room.progress.cleared
        && director.phase_timer > ROOM_CLEAR_GRACE_SECS
    {
        director.phase_timer = ROOM_CLEAR_GRACE_SECS;
        announcer.push_with_vo(
            &mut vo,
            VoKind::RoomClear,
            "Treasury Ghost: Objectives complete. Filing your heroism…",
        );
        play_juice(JuiceEvent::RoomClear, &mut shake, &mut flash, &mut audio);
    }

    // Meltdown fail forces the finale to end.
    if director.phase == TournamentPhase::Finale && room.failed && director.phase_timer > 0.5 {
        director.phase_timer = 0.5;
        announcer.push_with_vo(
            &mut vo,
            VoKind::MeltdownFail,
            "Treasury Ghost: Meltdown. The shuttle left without you. Mostly.",
        );
        play_juice(JuiceEvent::MeltdownFail, &mut shake, &mut flash, &mut audio);
    }

    director.phase_timer -= time.delta_secs();
    if director.phase_timer > 0.0 {
        return;
    }

    match director.phase {
        TournamentPhase::Lobby => {
            start_room(&mut director, &config, &mut room, &mut announcer, &mut vo)
        }
        TournamentPhase::RoomActive => end_room(
            &mut director,
            &config,
            &mut room,
            scoring.as_mut(),
            &mut announcer,
            ledger.as_mut(),
            &mut vo,
            mastery.as_mut(),
        ),
        TournamentPhase::Elimination => advance_after_elimination(
            &mut director,
            &config,
            &mut room,
            scoring.as_mut(),
            &mut announcer,
            &mut vo,
        ),
        TournamentPhase::Finale => end_finale(
            &mut director,
            &config,
            &mut room,
            scoring.as_mut(),
            &mut announcer,
            ledger.as_mut(),
            &mut vo,
            mastery.as_mut(),
        ),
        TournamentPhase::Podium => {
            director.phase = TournamentPhase::Complete;
            director.phase_timer = 0.0;
            announcer.push_with_vo(
                &mut vo,
                VoKind::Podium,
                "Podium: Corporate thanks you for your voluntary heroism.",
            );
        }
        TournamentPhase::Complete => {}
    }
}

fn start_room(
    director: &mut TournamentDirector,
    config: &TournamentConfig,
    room: &mut RoomRuntime,
    announcer: &mut AnnouncerQueue,
    vo: &mut crate::audio_fx::VoQueue,
) {
    director.phase = TournamentPhase::RoomActive;
    director.room = TournamentDirector::room_for_index(director.room_index);
    director.phase_timer = director.room.duration_secs(config.fast_timers);
    room.begin(director.room, director.alive_count(), config.slot_size);
    announcer.push_with_vo(
        vo,
        VoKind::RoomStart,
        format!(
            "Treasury Ghost: Welcome to {}. Compliance is mandatory and fun.",
            director.room.label()
        ),
    );
}

fn end_room(
    director: &mut TournamentDirector,
    config: &TournamentConfig,
    room: &mut RoomRuntime,
    scoring: &mut ScoringService,
    announcer: &mut AnnouncerQueue,
    ledger: &mut PracticeLedger,
    vo: &mut crate::audio_fx::VoQueue,
    mastery: &mut crate::meta::RoomMastery,
) {
    let cleared = room.progress.cleared;
    let clear_times = room.finish(config.slot_size);
    scoring.finalize_room(&clear_times);
    if cleared {
        mastery.record_clear(director.room);
    }

    let composite: Vec<(SlotId, f32)> = scoring
        .slot_composite
        .iter()
        .map(|(id, score)| (id.clone(), *score))
        .collect();

    let assign_strikes = director.room_index >= 1;
    director.apply_elimination(&composite, assign_strikes);

    if !director.danger_zone.is_empty() {
        announcer.push_with_vo(
            vo,
            VoKind::Elimination,
            "Treasury Ghost: Bottom slots, report to the Voluntary Separation Airlock.",
        );
    }

    director.phase = TournamentPhase::Elimination;
    director.phase_timer = ELIMINATION_DURATION_SECS;
    scoring.reset_room();

    if director.room_index >= 2 {
        ledger.accrue_practice_rewards(&director.placements);
    }

    let _ = config;
    let _ = room;
}

fn advance_after_elimination(
    director: &mut TournamentDirector,
    config: &TournamentConfig,
    room: &mut RoomRuntime,
    scoring: &mut ScoringService,
    announcer: &mut AnnouncerQueue,
    vo: &mut crate::audio_fx::VoQueue,
) {
    director.room_index += 1;

    // Play the full vault sequence unless the bracket collapses (< 2 alive).
    // (Previously `alive <= 3` skipped Cargo/Breaker on the default 4-slot local run.)
    if director.alive_count() < 2 || director.room_index >= 3 {
        director.phase = TournamentPhase::Finale;
        director.room = RoomId::ShuttleMeltdown;
        director.phase_timer = director.room.duration_secs(config.fast_timers);
        room.begin(director.room, director.alive_count(), config.slot_size);
        announcer.push_with_vo(
            vo,
            VoKind::RoomStart,
            "Treasury Ghost: Meltdown imminent. Heroism is voluntary.",
        );
        return;
    }

    for slot in &director.slots {
        if slot.alive {
            let players = (0..slot.size.player_count())
                .map(|p| PlayerScoreId(slot.id.0 * 10 + p as u32))
                .collect();
            scoring.register_slot(slot.id.clone(), players);
        }
    }

    start_room(director, config, room, announcer, vo);
}

fn end_finale(
    director: &mut TournamentDirector,
    config: &TournamentConfig,
    room: &mut RoomRuntime,
    scoring: &mut ScoringService,
    announcer: &mut AnnouncerQueue,
    ledger: &mut PracticeLedger,
    vo: &mut crate::audio_fx::VoQueue,
    mastery: &mut crate::meta::RoomMastery,
) {
    if room.progress.cleared {
        mastery.record_clear(director.room);
    }
    let clear_times = room.finish(config.slot_size);
    scoring.finalize_room(&clear_times);
    let composite: Vec<(SlotId, f32)> = scoring
        .slot_composite
        .iter()
        .map(|(id, score)| (id.clone(), *score))
        .collect();
    director.finalize_podium(&composite);

    let payouts = PayoutCalculator::top_three(
        config.match_mode,
        director.slots.len(),
        config.slot_size.player_count(),
    );
    ledger.apply_podium(payouts, &director.placements);
    announcer.push_with_vo(
        vo,
        VoKind::Podium,
        "Treasury Ghost: Payouts are taxable in 47 sectors.",
    );
    let _ = room;
}

fn bot_room_progress(
    time: Res<Time>,
    director: Res<TournamentDirector>,
    mut room: ResMut<RoomRuntime>,
    mut scoring: ResMut<ScoringService>,
    mut announcer: ResMut<AnnouncerQueue>,
    mut vo: ResMut<crate::audio_fx::VoQueue>,
) {
    if director.phase != TournamentPhase::RoomActive && director.phase != TournamentPhase::Finale {
        return;
    }

    if room.tick_ambient(time.delta_secs()) {
        announcer.push_with_vo(
            &mut vo,
            VoKind::MeltdownFail,
            "Treasury Ghost: Meltdown critical. Evacuation is… aspirational.",
        );
    }

    for slot in director.slots.iter().filter(|s| s.alive && s.is_bot) {
        room.tick_bot_slot(scoring.as_mut(), &slot.id, time.delta_secs());
    }
}

fn sync_tournament_snapshot(
    director: Res<TournamentDirector>,
    room: Res<RoomRuntime>,
    announcer: Res<AnnouncerQueue>,
    mut snapshots: Query<&mut TournamentSnapshot>,
) {
    let Ok(mut snap) = snapshots.single_mut() else {
        return;
    };
    snap.phase = director.phase;
    snap.room = director.room;
    snap.alive_slots = director.alive_count() as u8;
    snap.alive_mask = director
        .slots
        .iter()
        .enumerate()
        .fold(0u32, |mask, (i, slot)| {
            if slot.alive && i < 32 {
                mask | (1 << i)
            } else {
                mask
            }
        });
    snap.room_progress = room.progress_percent();
    snap.sort_target = room.sort_target;
    snap.meltdown_percent = room.meltdown_percent();
    snap.phase_timer = director.phase_timer;
    snap.announcer_line = announcer.last_bark.clone();
    snap.tournament_complete = director.phase == TournamentPhase::Complete;
}
