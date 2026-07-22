# Pudgy Monsters roster

Chunky party creatures for **PudgyMon: Party Saga**. One shared base, species skins that match the same proportions so movement and animations stay in sync.

## Base rig — `char_pudgy_base_01`

- Round body, oversized head, stubby limbs
- Soft rubber look; family-friendly
- Playable height ~1.2 m (read as “cute chunky,” not adult humanoid)
- GLB: `assets/models/char_pudgy_base_01/char_pudgy_base_01.glb`
- **Provisional mesh:** Ocean PudgyMon (Studio job) — first species also kept as `oceanic_pudgymon_01`

Default crew id: [`data/player_defaults.json`](../data/player_defaults.json). If the GLB is missing, runtime uses a **procedural Pudgy stub**.

## Pudgy Character Contract

All playable Pudgys (base + species skins) must obey this contract so one animation set can drive every variant without stretch/squash distortion.

| Rule | Value |
|------|--------|
| Base asset id | `char_pudgy_base_01` |
| Species id pattern | `char_pudgy_<biome>_01` or descriptive `*_pudgymon_01` |
| Playable height | ~1.2 m |
| Pivot | Floor center, +Y up, character faces **−Z** (Bevy forward) |
| Studio pose | Neutral **A-pose**, arms slightly out, feet planted — not a swim/run pose |
| Registry scale | Prefer `uniform_scale` (base uses `0.27`). Do **not** put Studio `target_height_m` (e.g. 4.5) straight into spawn scale |
| Required nodes (retarget target) | `Root`, `Hips`, `Spine`, `Head`, `L_Arm`, `R_Arm`, `L_Leg`, `R_Leg` |
| Shared clip names | `idle`, `walk`, `run`, `jump`, `emote_wave` |

**Tripo note:** exports often use a soft hierarchy. The node list above is the **retarget target** for future clips. Variants that do not match get root-transform motion only until retargeted.

**Import rule:** register species with the same `uniform_scale` as the base unless you measure a different mesh height.

## Species skins

| Id | Label | Notes |
|----|-------|--------|
| `oceanic_pudgymon_01` | Ocean PudgyMon | First species skin; same mesh/scale as provisional base |

Future biomes (forest, lava, sky, …) should change silhouette details and palette only — **same limb lengths and torso proportions** as the base.

## Starter color skins (season catalog)

Palette / accessory swaps on top of the equipped model (cycle with **C** in The Nest):

| Id | Label | Vibe |
|----|-------|------|
| `skin_starter` | Pudgy Sprout | Coral default |
| `skin_vibe` | Sunny Blob | Yellow party |
| `skin_racer` | Turbo Dumpling | Cyan speed |
| `skin_blaster` | Party Peep | Pink blaster |

## Nest showcase

Mannequins around The Nest preview each catalog tint. Unlock by season points, then claim on Boing (see [BOING_INTEGRATION.md](BOING_INTEGRATION.md)).

## Art pipeline

1. Shared base `char_pudgy_base_01` ← **current crew default**
2. Species variants via Studio using the species-variant prompt ([STUDIO_PROMPTS.md](STUDIO_PROMPTS.md))
3. Accessory / hat meshes per color skin (optional)
4. Shared clips (`idle` / `walk` / `run` / …) authored on the base, retargeted to matching species

## Runtime hook

```rust
PlayerVisualSpec {
    model_id: Some("char_pudgy_base_01".into()), // or a species asset_id
    hat_slot: slot % 8,
}
```

Default id: `data/player_defaults.json`. Cosmetics may later override `model_id` to a species skin that still matches this contract.
