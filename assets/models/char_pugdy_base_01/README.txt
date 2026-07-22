PugdyMon base character drop-in
================================

Drop your Studio / Tripo GLB here as:

  assets/models/char_pugdy_base_01/char_pugdy_base_01.glb

Then register (target height ~1.2 for chunky Pugdy proportions):

  python scripts/register_studio_asset.py char_pugdy_base_01 --height 1.2

`data/player_defaults.json` already points at this asset_id.
Until the GLB exists, the game uses a procedural Pugdy stub (round body + head).

Prompt reference: docs/STUDIO_PROMPTS.md → char_pugdy_base_01
