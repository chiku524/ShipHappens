//! Shared Pudgy clip playback — idle / walk / run + numbered emote slots.

use std::time::Duration;

use bevy::{
    animation::{transition::AnimationTransitions, AnimationPlayer},
    gltf::Gltf,
    prelude::*,
    world_serialization::WorldInstanceReady,
};

use crate::{flow::AppScreen, player::PlayerVisualRoot};

pub const CREW_CLIP_IDLE: &str = "idle";
pub const CREW_CLIP_WALK: &str = "walk";
pub const CREW_CLIP_RUN: &str = "run";

/// Performance clips eligible for keys 1–5 / Animations menu (not locomotion/idle).
const EMOTE_CANDIDATES: &[(&str, &str, f32, bool)] = &[
    // name in GLB, UI label, lock seconds (ignored when loops), loops
    ("jump", "Jump", 0.7, false),
    ("emote_scared", "Scared", 1.4, false),
    ("emote_wave", "Wave", 1.3, false),
    ("emote_dance", "Dance", 0.0, true),
    ("emote_cheer", "Cheer", 1.5, false),
];

pub const EMOTE_SLOT_COUNT: usize = 5;

const CROSSFADE: Duration = Duration::from_millis(150);
const WALK_SPEED_EPS: f32 = 0.05;

#[derive(Component, Debug, Clone)]
pub struct CrewAnimationSetup {
    pub model_id: String,
    pub gltf: Handle<Gltf>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrewAnimKind {
    #[default]
    Idle,
    Walk,
    Run,
    /// Index into [`CrewAnimPlayback::emotes`] (keys 1–5).
    Emote(u8),
}

impl CrewAnimKind {
    fn loops(self, playback: &CrewAnimPlayback) -> bool {
        match self {
            Self::Idle | Self::Walk | Self::Run => true,
            Self::Emote(i) => playback
                .emotes
                .get(i as usize)
                .map(|e| e.loops)
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrewEmoteSlot {
    pub node: AnimationNodeIndex,
    pub label: String,
    pub clip_name: String,
    pub lock_secs: f32,
    pub loops: bool,
}

#[derive(Component, Debug)]
pub struct CrewAnimPlayback {
    pub kind: CrewAnimKind,
    pub applied: Option<CrewAnimKind>,
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub emotes: Vec<CrewEmoteSlot>,
    pub lock_until: f32,
    pub player_entity: Entity,
}

impl CrewAnimPlayback {
    fn node(&self, kind: CrewAnimKind) -> AnimationNodeIndex {
        match kind {
            CrewAnimKind::Idle => self.idle,
            CrewAnimKind::Walk => self.walk,
            CrewAnimKind::Run => self.run,
            CrewAnimKind::Emote(i) => self
                .emotes
                .get(i as usize)
                .map(|e| e.node)
                .unwrap_or(self.idle),
        }
    }

    pub fn trigger_emote(&mut self, slot: u8, now: f32) {
        let Some(emote) = self.emotes.get(slot as usize) else {
            return;
        };
        self.kind = CrewAnimKind::Emote(slot);
        self.lock_until = if emote.loops {
            f32::MAX
        } else {
            now + emote.lock_secs
        };
        self.applied = None;
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerMotion {
    pub speed: f32,
    pub sprint: bool,
}

/// Queued from the Nest Animations menu (plays on the local crew mesh).
#[derive(Message, Debug, Clone, Copy)]
pub struct PlayCrewEmote(pub u8);

#[derive(Component)]
struct CrewSceneReady;

pub struct CrewAnimationPlugin;

impl Plugin for CrewAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayCrewEmote>().add_systems(
            Update,
            (
                recover_stale_crew_playback,
                poll_crew_scene_ready,
                finish_crew_animation_setup,
                choose_crew_anim_kind.run_if(in_state(AppScreen::Playing)),
                apply_crew_anim_kind.run_if(in_state(AppScreen::Playing)),
            )
                .chain(),
        );
    }
}

/// Attach after spawning `WorldAssetRoot` (prefer spawning setup+observe together).
pub fn attach_crew_animation(
    entity: Entity,
    commands: &mut Commands,
    asset_server: &AssetServer,
    model_id: &str,
) {
    let glb_path = format!("models/{model_id}/{model_id}.glb");
    let gltf: Handle<Gltf> = asset_server.load(glb_path);
    commands
        .entity(entity)
        .insert(CrewAnimationSetup {
            model_id: model_id.to_string(),
            gltf,
        })
        .observe(on_crew_scene_ready);
}

pub fn on_crew_scene_ready(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    setups: Query<&CrewAnimationSetup>,
) {
    if !setups.contains(ready.entity) {
        return;
    }
    // WorldAsset may respawn when dependencies finish loading (AssetEvent::Modified).
    // Drop any prior binding so we re-attach to the new AnimationPlayer.
    commands
        .entity(ready.entity)
        .remove::<CrewAnimPlayback>()
        .insert(CrewSceneReady);
}

/// If the skinned instance was respawned, `player_entity` goes stale — force a rebind.
fn recover_stale_crew_playback(
    mut commands: Commands,
    playback: Query<(Entity, &CrewAnimPlayback), With<CrewAnimationSetup>>,
    players: Query<(), With<AnimationPlayer>>,
) {
    for (entity, anim) in &playback {
        if !players.contains(anim.player_entity) {
            commands
                .entity(entity)
                .remove::<CrewAnimPlayback>()
                .insert(CrewSceneReady);
        }
    }
}

/// Fallback when `WorldInstanceReady` was missed (observe registered too late).
fn poll_crew_scene_ready(
    mut commands: Commands,
    pending: Query<
        Entity,
        (
            With<CrewAnimationSetup>,
            With<WorldAssetRoot>,
            Without<CrewSceneReady>,
            Without<CrewAnimPlayback>,
        ),
    >,
    children: Query<&Children>,
    players: Query<(), With<AnimationPlayer>>,
) {
    for entity in &pending {
        let has_player = players.contains(entity)
            || children
                .iter_descendants(entity)
                .any(|desc| players.contains(desc));
        if has_player {
            commands.entity(entity).insert(CrewSceneReady);
        }
    }
}

fn finish_crew_animation_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    pending: Query<
        (Entity, &CrewAnimationSetup),
        (With<CrewSceneReady>, Without<CrewAnimPlayback>),
    >,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    for (entity, setup) in &pending {
        if !asset_server.is_loaded_with_dependencies(&setup.gltf) {
            continue;
        }
        let Some(gltf) = gltfs.get(&setup.gltf) else {
            continue;
        };

        let Some(idle) = gltf.named_animations.get(CREW_CLIP_IDLE).cloned() else {
            warn!(
                "crew GLB `{}` missing clip `{CREW_CLIP_IDLE}` — keys: {:?}",
                setup.model_id,
                gltf.named_animations.keys().collect::<Vec<_>>()
            );
            commands
                .entity(entity)
                .remove::<(CrewSceneReady, CrewAnimationSetup)>();
            continue;
        };
        let walk = named_or(gltf, CREW_CLIP_WALK, &idle);
        let run = named_or(gltf, CREW_CLIP_RUN, &walk);

        let mut clip_handles = vec![idle.clone(), walk, run];
        let mut emote_defs: Vec<(String, String, f32, bool)> = Vec::new();
        for &(clip_name, label, lock_secs, loops) in EMOTE_CANDIDATES {
            if let Some(handle) = gltf.named_animations.get(clip_name).cloned() {
                clip_handles.push(handle);
                emote_defs.push((clip_name.to_string(), label.to_string(), lock_secs, loops));
            }
        }
        // Fill remaining 1–5 slots with idle so keybinds always resolve.
        while emote_defs.len() < EMOTE_SLOT_COUNT {
            emote_defs.push((
                CREW_CLIP_IDLE.to_string(),
                format!("Emote {}", emote_defs.len() + 1),
                1.0,
                false,
            ));
            clip_handles.push(idle.clone());
        }
        emote_defs.truncate(EMOTE_SLOT_COUNT);

        let (graph, nodes) = AnimationGraph::from_clips(clip_handles);
        let graph_handle = graphs.add(graph);

        let mut player_entity = None;
        if players.contains(entity) {
            player_entity = Some(entity);
        } else {
            for desc in children.iter_descendants(entity) {
                if players.contains(desc) {
                    player_entity = Some(desc);
                    break;
                }
            }
        }
        let Some(player_entity) = player_entity else {
            continue;
        };

        let Ok(mut player) = players.get_mut(player_entity) else {
            continue;
        };
        let mut transitions = AnimationTransitions::new();
        transitions
            .play(&mut player, nodes[0], Duration::ZERO)
            .repeat();

        commands.entity(player_entity).insert((
            AnimationGraphHandle(graph_handle.clone()),
            transitions,
        ));

        let emotes: Vec<CrewEmoteSlot> = emote_defs
            .into_iter()
            .enumerate()
            .map(|(i, (clip_name, label, lock_secs, loops))| CrewEmoteSlot {
                node: nodes[3 + i],
                label,
                clip_name,
                lock_secs,
                loops,
            })
            .collect();

        info!(
            "crew animation ready for `{}` (player={player_entity:?}, emotes={:?})",
            setup.model_id,
            emotes.iter().map(|e| e.clip_name.as_str()).collect::<Vec<_>>()
        );

        commands.entity(entity).insert(CrewAnimPlayback {
            kind: CrewAnimKind::Idle,
            applied: Some(CrewAnimKind::Idle),
            graph: graph_handle,
            idle: nodes[0],
            walk: nodes[1],
            run: nodes[2],
            emotes,
            lock_until: 0.0,
            player_entity,
        });
        commands.entity(entity).remove::<CrewSceneReady>();
    }
}

fn named_or(gltf: &Gltf, name: &str, fallback: &Handle<AnimationClip>) -> Handle<AnimationClip> {
    gltf.named_animations
        .get(name)
        .cloned()
        .unwrap_or_else(|| fallback.clone())
}

fn choose_crew_anim_kind(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    pause: Option<Res<crate::settings::PauseState>>,
    mut emote_events: MessageReader<PlayCrewEmote>,
    players: Query<(&PlayerMotion, &Children, Option<&crate::player::LocalPlayer>)>,
    visual_roots: Query<(), With<PlayerVisualRoot>>,
    mut visuals: Query<&mut CrewAnimPlayback>,
) {
    let paused = pause.map(|p| p.paused).unwrap_or(false);
    let now = time.elapsed_secs();

    // Menu-triggered emotes apply to the local player even while the pause UI is open.
    let menu_emotes: Vec<u8> = emote_events.read().map(|e| e.0).collect();

    for (motion, children, local) in &players {
        let Some(visual) = children.iter().find(|c| visual_roots.contains(*c)) else {
            continue;
        };
        let Ok(mut anim) = visuals.get_mut(visual) else {
            continue;
        };

        if local.is_some() {
            for slot in &menu_emotes {
                anim.trigger_emote(*slot, now);
            }
        }

        let moving = motion.speed > WALK_SPEED_EPS;
        let locked = now < anim.lock_until;

        if local.is_some() && !paused {
            if keyboard.just_pressed(KeyCode::Space) {
                if let Some(slot) = emote_slot_by_clip(&anim, "jump") {
                    anim.trigger_emote(slot, now);
                    continue;
                }
            }
            let digit_slot = [
                (KeyCode::Digit1, 0u8),
                (KeyCode::Digit2, 1),
                (KeyCode::Digit3, 2),
                (KeyCode::Digit4, 3),
                (KeyCode::Digit5, 4),
                (KeyCode::Numpad1, 0),
                (KeyCode::Numpad2, 1),
                (KeyCode::Numpad3, 2),
                (KeyCode::Numpad4, 3),
                (KeyCode::Numpad5, 4),
            ]
            .into_iter()
            .find_map(|(key, slot)| keyboard.just_pressed(key).then_some(slot));
            if let Some(slot) = digit_slot {
                anim.trigger_emote(slot, now);
                continue;
            }
        }

        if locked {
            if let CrewAnimKind::Emote(i) = anim.kind {
                let loops = anim.emotes.get(i as usize).map(|e| e.loops).unwrap_or(false);
                if loops {
                    if moving {
                        anim.lock_until = 0.0;
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        }

        let desired = if !moving {
            CrewAnimKind::Idle
        } else if motion.sprint {
            CrewAnimKind::Run
        } else {
            CrewAnimKind::Walk
        };
        if anim.kind != desired {
            anim.kind = desired;
            anim.applied = None;
        }
    }
}

fn emote_slot_by_clip(anim: &CrewAnimPlayback, clip: &str) -> Option<u8> {
    anim.emotes
        .iter()
        .position(|e| e.clip_name == clip)
        .map(|i| i as u8)
}

fn apply_crew_anim_kind(
    mut commands: Commands,
    mut visuals: Query<(Entity, &mut CrewAnimPlayback)>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (entity, mut anim) in &mut visuals {
        if anim.applied == Some(anim.kind) {
            continue;
        }
        let Ok((mut player, mut transitions)) = players.get_mut(anim.player_entity) else {
            // Instance was respawned — recover next frame.
            commands
                .entity(entity)
                .remove::<CrewAnimPlayback>()
                .insert(CrewSceneReady);
            continue;
        };
        let node = anim.node(anim.kind);
        let active = transitions.play(&mut player, node, CROSSFADE);
        if anim.kind.loops(&anim) {
            active.repeat();
        }
        anim.applied = Some(anim.kind);
    }
}
