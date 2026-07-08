use std::{collections::HashMap, fs, path::Path};

use bevy::prelude::*;
use serde::Deserialize;

use crate::tournament::types::RoomId;

pub const ROOM_LAYOUT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RoomLayoutDefinition {
    pub schema_version: u32,
    pub room_id: String,
    pub label: String,
    #[serde(default = "default_player_spawn")]
    pub player_spawn: [f32; 3],
    pub markers: Vec<LayoutMarker>,
}

fn default_player_spawn() -> [f32; 3] {
    [0.0, 1.0, 8.0]
}

/// One placement slot in a vault stage. Set `asset_id` when the Tripo GLB is ready;
/// until then the greybox placeholder (if defined) shows the intended footprint.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LayoutMarker {
    pub id: String,
    pub role: MarkerRole,
    #[serde(default)]
    pub asset_id: Option<String>,
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation_y_deg: f32,
    #[serde(default)]
    pub interactable: Option<InteractableSpec>,
    #[serde(default)]
    pub greybox: Option<GreyboxSpec>,
    /// Optional interact animation override. Omit to use station-kind defaults.
    #[serde(default)]
    pub motion: Option<MarkerMotionSpec>,
}

/// Per-marker motion tuning. `glb_clip` is reserved for future skeletal clips on Tripo meshes.
#[derive(Debug, Clone, Deserialize, PartialEq, Default)]
pub struct MarkerMotionSpec {
    /// Procedural preset on success. Use `"none"` to disable motion for this marker.
    #[serde(default)]
    pub preset: Option<MotionPresetKind>,
    /// Procedural preset on failed interact (wrong breaker, wrong chute).
    #[serde(default)]
    pub fail_preset: Option<MotionPresetKind>,
    /// Override animation length in seconds.
    #[serde(default)]
    pub duration_secs: Option<f32>,
    /// Future: name of GLB animation clip to play (e.g. Mixamo / Tripo rig).
    #[serde(default)]
    pub glb_clip: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MotionPresetKind {
    None,
    BreakerFlip,
    BreakerZap,
    ValveTurn,
    DoorSeal,
    SortChutePulse,
    VaultBob,
    CraneNudge,
    ZonePulse,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarkerRole {
    Station,
    Decoration,
    Zone,
    Sign,
    SortChute,
    FloorVfx,
    Floor,
    FloorDetail,
    Wall,
    Ceiling,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ArenaLayoutDefinition {
    pub schema_version: u32,
    pub label: String,
    #[serde(default = "default_player_spawn")]
    pub lobby_spawn: [f32; 3],
    pub markers: Vec<LayoutMarker>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InteractableSpec {
    Crane,
    VaultObjective,
    SortChute { index: u8 },
    Breaker { index: u8 },
    CoolantValve { index: u8 },
    MeltdownDoor { index: u8 },
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GreyboxSpec {
    pub size: [f32; 3],
    pub color: [f32; 3],
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub emissive: Option<[f32; 3]>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RoomLayoutCatalog {
    layouts: HashMap<RoomId, RoomLayoutDefinition>,
}

impl RoomLayoutCatalog {
    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, RoomLayoutError> {
        let dir = dir.as_ref();
        let mut layouts = HashMap::new();

        for room in [
            RoomId::HrOrientation,
            RoomId::CargoGantry,
            RoomId::BreakerPanic,
            RoomId::ShuttleMeltdown,
        ] {
            let path = dir.join(format!("{}.json", room.file_stem()));
            let layout = load_room_layout(&path)?;
            if layout.room_id != room.file_stem() {
                return Err(RoomLayoutError::Parse(format!(
                    "{}: room_id `{}` does not match file `{}`",
                    path.display(),
                    layout.room_id,
                    room.file_stem()
                )));
            }
            if layout.schema_version != ROOM_LAYOUT_SCHEMA_VERSION {
                return Err(RoomLayoutError::Parse(format!(
                    "{}: unsupported schema_version {}",
                    path.display(),
                    layout.schema_version
                )));
            }
            layouts.insert(room, layout);
        }

        Ok(Self { layouts })
    }

    pub fn get(&self, room: RoomId) -> Option<&RoomLayoutDefinition> {
        self.layouts.get(&room)
    }

    pub fn len(&self) -> usize {
        self.layouts.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomLayoutError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for RoomLayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "failed to read room layout: {msg}"),
            Self::Parse(msg) => write!(f, "failed to parse room layout: {msg}"),
        }
    }
}

impl std::error::Error for RoomLayoutError {}

pub fn load_room_layout(path: impl AsRef<Path>) -> Result<RoomLayoutDefinition, RoomLayoutError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|err| RoomLayoutError::Io(err.to_string()))?;
    serde_json::from_str(&raw).map_err(|err| RoomLayoutError::Parse(err.to_string()))
}

pub fn load_arena_layout(path: impl AsRef<Path>) -> Result<ArenaLayoutDefinition, RoomLayoutError> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path).map_err(|err| RoomLayoutError::Io(err.to_string()))?;
    let layout: ArenaLayoutDefinition = serde_json::from_str(&raw)
        .map_err(|err| RoomLayoutError::Parse(err.to_string()))?;
    if layout.schema_version != ROOM_LAYOUT_SCHEMA_VERSION {
        return Err(RoomLayoutError::Parse(format!(
            "{}: unsupported schema_version {}",
            path.display(),
            layout.schema_version
        )));
    }
    Ok(layout)
}

#[derive(Resource, Debug, Clone)]
pub struct ArenaLayout(pub ArenaLayoutDefinition);

impl RoomId {
    pub fn file_stem(self) -> &'static str {
        match self {
            Self::HrOrientation => "hr_orientation",
            Self::CargoGantry => "cargo_gantry",
            Self::BreakerPanic => "breaker_panic",
            Self::ShuttleMeltdown => "shuttle_meltdown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_room_layouts() {
        let catalog =
            RoomLayoutCatalog::load_from_dir("data/rooms").expect("room layouts load");
        assert_eq!(catalog.len(), 4);
        let hr = catalog.get(RoomId::HrOrientation).expect("hr layout");
        assert!(hr.markers.iter().any(|m| m.id == "sort_chute_hot_dogs"));
    }

    #[test]
    fn loads_arena_layout() {
        let arena = load_arena_layout("data/rooms/arena.json").expect("arena loads");
        assert!(arena.markers.iter().any(|m| m.id == "floor_main"));
    }

    #[test]
    fn parses_marker_motion_spec() {
        let json = r#"{
            "id": "test_breaker",
            "role": "station",
            "position": [0,0,0],
            "motion": {
                "preset": "breaker_flip",
                "fail_preset": "breaker_zap",
                "duration_secs": 0.4,
                "glb_clip": "switch_flip"
            }
        }"#;
        let marker: LayoutMarker = serde_json::from_str(json).expect("marker parses");
        let motion = marker.motion.expect("motion present");
        assert_eq!(motion.preset, Some(MotionPresetKind::BreakerFlip));
        assert_eq!(motion.fail_preset, Some(MotionPresetKind::BreakerZap));
        assert_eq!(motion.duration_secs, Some(0.4));
        assert_eq!(motion.glb_clip.as_deref(), Some("switch_flip"));
    }
}
