# Asset wishlist

Stand-in assets are already placed so rooms stay playable. Replace these when dedicated Tripo meshes land — **only change `asset_id` (and maybe `scale`) in the room JSON**; keep the same marker `id`.

**Copy-paste Tripo / Immersive Studio prompts:** [STUDIO_PROMPTS.md](STUDIO_PROMPTS.md)

| Priority | Suggested `asset_id` | Replace marker(s) | Current stand-in |
|----------|----------------------|-------------------|------------------|
| High | `env_sort_chute_hot_dogs_01` (×4 variants or one reusable) | `sort_chute_*` in `hr_orientation.json` | `prop_pneumatic_tube_intake_funnel` |
| High | `env_shuttle_seal_door_01` | `meltdown_door_*` in `shuttle_meltdown.json` | `env_break_glass_panel_01` |
| High | `char_pugdy_base_01` | `PlayerVisualSpec.model_id` | Procedural Pugdy stub |
| High | `env_nest_egg_01` | Nest centerpiece | Greybox sphere |
| Medium | `env_nest_bench_01` | Nest seating | Greybox benches |
| Medium | `prop_vibe_mushroom_01` | Nest flora | Greybox mushrooms |
| Legacy | `char_crew_base_01` | retired | — |
| Medium | `env_freight_deck_panel_01` | `floor_main` in `arena.json` | Greybox slab |
| Medium | `env_room_sign_*` per vault | `room_sign` (HR / Breaker / Meltdown) | `env_blue_yellow_welcome_sign_01` |
| Medium | `vfx_meltdown_floor_glow_01` | `meltdown_glow` | Emissive greybox plane |
| Low | Extra breaker panels (GDD wants 12) | new markers in `breaker_panic.json` | — |
| Low | Extra coolant valves (GDD wants 6) | new markers in `shuttle_meltdown.json` | — |
| Low | Hat meshes `char_hat_zip_01` … | child of character | — |

## Drop-in checklist

1. Generate + import pack (`scripts/import_immersive_studio_pack.py`)
2. Confirm GLB at `assets/models/<id>/<id>.glb`
3. Swap `asset_id` on the marker(s) above
4. `python scripts/validate_studio_assets.py`
5. `cargo run -- local` and check scale / pivot
