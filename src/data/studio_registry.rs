use std::{fs, path::Path};

use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct StudioAssetEntry {
    pub asset_id: String,
    #[serde(default = "default_target_height")]
    pub target_height: f32,
}

fn default_target_height() -> f32 {
    1.0
}

#[derive(Debug, Deserialize)]
struct StudioRegistryFile {
    import_root: String,
    assets: Vec<StudioAssetEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudioRegistryError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for StudioRegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "failed to read studio registry: {msg}"),
            Self::Parse(msg) => write!(f, "failed to parse studio registry: {msg}"),
        }
    }
}

impl std::error::Error for StudioRegistryError {}

#[derive(Resource, Debug, Clone)]
pub struct StudioRegistry {
    pub import_root: String,
    pub assets: Vec<StudioAssetEntry>,
}

impl StudioRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, StudioRegistryError> {
        let raw = fs::read_to_string(path.as_ref())
            .map_err(|err| StudioRegistryError::Io(err.to_string()))?;
        let parsed: StudioRegistryFile = serde_json::from_str(&raw)
            .map_err(|err| StudioRegistryError::Parse(err.to_string()))?;
        Ok(Self {
            import_root: parsed.import_root,
            assets: parsed.assets,
        })
    }

    pub fn target_height(&self, asset_id: &str) -> Option<f32> {
        self.assets
            .iter()
            .find(|entry| entry.asset_id == asset_id)
            .map(|entry| entry.target_height)
    }

    pub fn glb_asset_path(&self, asset_id: &str) -> String {
        format!("models/{asset_id}/{asset_id}.glb")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_studio_registry() {
        let registry =
            StudioRegistry::load("assets/studio_registry.json").expect("registry loads");
        assert!(registry.target_height("env_breaker_panel_01").is_some());
    }
}
