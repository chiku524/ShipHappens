//! Cosmetic catalog + equipped skin.

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{network::OwnedPlayer, player::PlayerColor, season::SeasonLedger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmeticItem {
    pub id: String,
    pub label: String,
    pub cost_points: u32,
    pub tint: [f32; 3],
    pub boing_token_id: Option<u64>,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CosmeticsCatalog {
    pub items: Vec<CosmeticItem>,
}

impl Default for CosmeticsCatalog {
    fn default() -> Self {
        Self {
            items: vec![
                CosmeticItem {
                    id: "skin_starter".into(),
                    label: "Pugdy Sprout".into(),
                    cost_points: 0,
                    tint: [0.95, 0.45, 0.35],
                    boing_token_id: Some(1),
                },
                CosmeticItem {
                    id: "skin_vibe".into(),
                    label: "Sunny Blob".into(),
                    cost_points: 50,
                    tint: [1.0, 0.85, 0.25],
                    boing_token_id: Some(2),
                },
                CosmeticItem {
                    id: "skin_racer".into(),
                    label: "Turbo Dumpling".into(),
                    cost_points: 120,
                    tint: [0.25, 0.8, 0.95],
                    boing_token_id: Some(3),
                },
                CosmeticItem {
                    id: "skin_blaster".into(),
                    label: "Party Peep".into(),
                    cost_points: 200,
                    tint: [1.0, 0.4, 0.65],
                    boing_token_id: Some(4),
                },
            ],
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct EquippedCosmetic {
    pub id: String,
}

impl Default for EquippedCosmetic {
    fn default() -> Self {
        Self {
            id: "skin_starter".into(),
        }
    }
}

#[derive(Event, Serialize, Deserialize, Clone, Debug)]
pub struct EquipCosmeticRequest {
    pub id: String,
    pub tint: [f32; 3],
}

pub struct CosmeticsPlugin;

impl Plugin for CosmeticsPlugin {
    fn build(&self, app: &mut App) {
        let catalog = load_catalog();
        app.insert_resource(catalog)
            .init_resource::<EquippedCosmetic>()
            .add_client_event::<EquipCosmeticRequest>(Channel::Unordered)
            .add_observer(handle_equip_cosmetic)
            .add_systems(Update, cycle_equipped_skin);
    }
}

fn load_catalog() -> CosmeticsCatalog {
    let path = format!(
        "{}/data/cosmetics/catalog.json",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn cycle_equipped_skin(
    keyboard: Res<ButtonInput<KeyCode>>,
    ledger: Res<SeasonLedger>,
    catalog: Res<CosmeticsCatalog>,
    mut equipped: ResMut<EquippedCosmetic>,
    mut commands: Commands,
    mut colors: Query<&mut PlayerColor, With<crate::player::LocalPlayer>>,
    client: Option<Res<bevy_replicon_renet::RenetClient>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyC) {
        return;
    }
    let unlocked: Vec<&CosmeticItem> = catalog
        .items
        .iter()
        .filter(|i| ledger.unlocked.contains(&i.id))
        .collect();
    if unlocked.is_empty() {
        return;
    }
    let idx = unlocked
        .iter()
        .position(|i| i.id == equipped.id)
        .unwrap_or(0);
    let next = unlocked[(idx + 1) % unlocked.len()];
    equipped.id = next.id.clone();
    if client.is_some() {
        commands.client_trigger(EquipCosmeticRequest {
            id: next.id.clone(),
            tint: next.tint,
        });
    } else if let Ok(mut color) = colors.single_mut() {
        color.0 = next.tint;
    }
}

fn handle_equip_cosmetic(
    request: On<FromClient<EquipCosmeticRequest>>,
    mut players: Query<&mut PlayerColor, With<crate::player::NetworkPlayer>>,
    owners: Query<&OwnedPlayer>,
) {
    let Some(client_entity) = request.client_id.entity() else {
        return;
    };
    let Ok(owned) = owners.get(client_entity) else {
        return;
    };
    if let Ok(mut color) = players.get_mut(owned.0) {
        color.0 = request.tint;
    }
}
