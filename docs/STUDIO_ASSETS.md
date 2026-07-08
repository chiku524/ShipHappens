# Immersive Studio → ShipHappens workflow

ShipHappens uses [Immersive Labs Studio](https://github.com/chiku524/immersive.labs) to generate **Tripo AI** meshes with baked PBR, then imports GLBs into `assets/models/` for the Bevy runtime.

## Prerequisites

1. **Immersive Studio desktop** v0.1.7+ (or local worker) with `STUDIO_TRIPO_API_KEY` set — see `scripts/studio/worker.env.example`.
2. Python 3 for the import script.

## Generate a pack (Studio)

1. Open Immersive Studio and run a job with **Tripo textures** and **Generate 3D mesh** enabled.
2. Download `pack.zip` when complete.
3. In `pack_diagnostics.json`, confirm Tripo mesh + textures succeeded (not Comfy-only sidecars).

## Import into ShipHappens

From the repo root:

```bash
python scripts/import_immersive_studio_pack.py path/to/pack.zip
```

With `--update` to refresh `target_height` for assets already in the registry.

The script:

- Copies `Models/<asset_id>/` → `assets/models/<asset_id>/`
- Optionally copies `Textures/<asset_id>/` sidecars (skip with `--no-textures` when Tripo baked PBR into the GLB)
- Merges entries into `assets/studio_registry.json`

## Runtime wiring (Bevy)

| Piece | Location |
|-------|----------|
| Asset registry JSON | `assets/studio_registry.json` |
| GLB loader | `src/assets/mod.rs` → `StudioRegistry::load()` |
| Model path | `assets/models/{asset_id}/{asset_id}.glb` |
| Greybox level | `src/world/mod.rs` spawns registered props at station markers |

After importing a new asset, add it to the greybox level or job station spawn logic in `src/world/mod.rs` / `src/interaction/mod.rs`.

## Regenerating existing assets with Tripo

Older packs may use ComfyUI sidecars or placeholder meshes. Re-run Studio jobs with the same `asset_id`, import with `--update`, and re-test scale/placement in the Bevy level.
