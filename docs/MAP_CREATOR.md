# Map Creator — Race / Vibe / Shooter + Party Saga packs

In-game UGC for **PudgyMon: Party Saga**. Create layouts in The Nest, save locally, playtest, export share codes.

Playable XZ half-extent is `ARENA_BOUNDS` (**36**) — Nest floor is ~**72×72**. Place props anywhere inside ±36.

## Loop

1. Nest → orange **Create Map** → **E**
2. **Tab** switch layer: Race · Vibe · Shooter
3. Tools: **1** primary (gate / orb / cover) · **2** spawn · **3** block · **4** deco GLB (**,** / **.** cycle)
4. **F** / LMB place · **X** delete
5. **F5** save current layer · **F8** save full Party Saga pack
6. **F6** playtest layer · **F9** playtest full Party Saga
7. **F7** export share code → `%LOCALAPPDATA%/PudgyMon/maps/shares/`
8. Purple **My Maps** → **[ ]** cycle Race / Vibe / Shooter / packs · **E** play

Companion import desk: [`companion/maps/`](../companion/maps/index.html)

## Formats

### Race (`mode: "race"`)

`spawns`, `gates` (≥2), `blocks` (optional `asset_id`)

### Vibe (`mode: "vibe"`)

`spawns`, `orbs` (≥3), `blocks`

### Shooter (`mode: "shooter"`)

`spawns`, `cover` blocks (optional `asset_id`)

### Party Saga pack (`kind: "party_saga"`, schema 2)

```json
{
  "schema_version": 2,
  "id": "my_pack",
  "label": "My Party Saga",
  "kind": "party_saga",
  "author": "local",
  "race": { "...": "RaceMap" },
  "vibe": { "...": "VibeMap" },
  "shooter": { "...": "ShooterMap" }
}
```

Bundled: `official_race_loop.json`, `official_vibe_ring.json`, `official_shooter_yard.json`, `official_party_saga.json`

## Deco GLB palette

Key **4** places blocks tagged with studio `asset_id` from `EDITOR_DECO_IDS` (crates, vending, etc.) when the GLB exists under `assets/models/`. Runtime stages still spawn greybox collision volumes for MVP; full GLB stage spawn is next polish.

## Code

| Module | Role |
|--------|------|
| [`src/maps/`](../src/maps/) | Race/Vibe/Shooter/Pack types, catalog, share codes |
| [`src/map_editor/`](../src/map_editor/) | Multi-layer editor |
| [`src/stages/`](../src/stages/) | Boots from `ActiveStageMaps` |

## Later

- Live GLB meshes in stage boot for deco `asset_id`
- Online workshop / Boing-linked upload
- Co-edit / moderation
