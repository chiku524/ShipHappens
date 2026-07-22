//! Default crew model / hat hooks — swap when Studio GLBs land.

use std::{fs, path::Path};

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Debug, Clone)]
pub struct PlayerDefaults {
    pub crew_model_id: String,
}

impl Default for PlayerDefaults {
    fn default() -> Self {
        Self {
            crew_model_id: "char_pugdy_base_01".into(),
        }
    }
}

#[derive(Deserialize)]
struct PlayerDefaultsFile {
    crew_model_id: String,
}

impl PlayerDefaults {
    pub fn load(path: impl AsRef<Path>) -> Self {
        let Ok(raw) = fs::read_to_string(path.as_ref()) else {
            return Self::default();
        };
        match serde_json::from_str::<PlayerDefaultsFile>(&raw) {
            Ok(file) => Self {
                crew_model_id: file.crew_model_id,
            },
            Err(err) => {
                warn!("player_defaults.json parse failed: {err}");
                Self::default()
            }
        }
    }

    /// Returns crew model id when the GLB exists on disk (registry optional for scale).
    pub fn resolved_crew_model(&self) -> Option<String> {
        let path = format!(
            "{}/assets/models/{}/{}.glb",
            env!("CARGO_MANIFEST_DIR"),
            self.crew_model_id,
            self.crew_model_id
        );
        std::path::Path::new(&path)
            .is_file()
            .then(|| self.crew_model_id.clone())
    }
}

pub fn load_player_defaults(mut commands: Commands) {
    let path = format!("{}/data/player_defaults.json", env!("CARGO_MANIFEST_DIR"));
    commands.insert_resource(PlayerDefaults::load(path));
}
