# Drop-in guide (art + audio)

Wired so you can drop files and play — **no Rust required** for normal replacements.

Related: [CHARACTERS.md](CHARACTERS.md) · [STUDIO_ASSETS.md](STUDIO_ASSETS.md) · [STUDIO_PROMPTS.md](STUDIO_PROMPTS.md) · [ASSET_WISHLIST.md](ASSET_WISHLIST.md)

---

## Pugdy character (priority)

| What | Drop path | Then |
|------|-----------|------|
| Base Pugdy | `assets/models/char_pugdy_base_01/char_pugdy_base_01.glb` | `python scripts/register_studio_asset.py char_pugdy_base_01 --height 1.2` |
| Nest egg | `assets/models/env_nest_egg_01/env_nest_egg_01.glb` | Register; later wire into Nest spawn |
| Nest bench | `assets/models/env_nest_bench_01/…` | Same |
| Vibe mushroom | `assets/models/prop_vibe_mushroom_01/…` | Same |

Until the Pugdy GLB exists, players use a **procedural stub** (round body + head).

Validate:

```bash
python scripts/validate_studio_assets.py
cargo test
cargo run
```

---

## Audio

| Kind | Folder | Fallback |
|------|--------|----------|
| SFX | `assets/audio/sfx/` | Pitch tones |
| Music | `assets/audio/music/` | Silence |
| VO | `assets/audio/vo/` | HUD text |

Restart after dropping clips.

---

## Quick checklist

1. [ ] `char_pugdy_base_01.glb` → auto-swap from stub  
2. [ ] Nest props (`env_nest_egg_01`, benches, mushrooms)  
3. [ ] Music bed for The Nest / Party Saga  
4. [ ] SFX: pickup / KO / finish  
5. [ ] Open claim companion (`Ctrl+O`) after minting path is live  
