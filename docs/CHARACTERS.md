# Pudgy Monsters roster

Chunky party creatures for **PudgyMon: Party Saga**. One base body, palette + accessory swaps.

## Base rig — `char_pudgy_base_01`

- Round body, oversized head, stubby limbs
- Soft rubber look; family-friendly
- Target height ~1.2m (read as “cute chunky,” not adult humanoid)
- Drop GLB at `assets/models/char_pudgy_base_01/char_pudgy_base_01.glb`

Until the GLB lands, runtime uses a **procedural Pudgy stub** (sphere body + head + eyes).

## Starter skins (season catalog)

| Id | Label | Vibe |
|----|-------|------|
| `skin_starter` | Pudgy Sprout | Coral default |
| `skin_vibe` | Sunny Blob | Yellow party |
| `skin_racer` | Turbo Dumpling | Cyan speed |
| `skin_blaster` | Party Peep | Pink blaster |

Cycle unlocked skins with **C** in The Nest.

## Nest showcase

Mannequins around The Nest preview each catalog tint. Unlock by season points, then claim on Boing (see [BOING_INTEGRATION.md](BOING_INTEGRATION.md)).

## Art pipeline

1. Procedural Pudgy stub ← **current default**
2. Drop `char_pudgy_base_01.glb` (auto-swap via `PlayerVisualSpec`)
3. Accessory / hat meshes per skin (optional)
4. Idle / run Mixamo or custom clips

## Runtime hook

```rust
PlayerVisualSpec {
    model_id: Some("char_pudgy_base_01".into()), // when GLB exists
    hat_slot: slot % 8,
}
```

Default id: `data/player_defaults.json`.
