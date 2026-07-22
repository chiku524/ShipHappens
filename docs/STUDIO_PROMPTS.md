# Immersive Studio prompt pack — PudgyMon

Copy-paste prompts for [Immersive Labs Studio](https://github.com/chiku524/immersive.labs) / Tripo jobs. Each entry includes the **`asset_id`**, target height, where it plugs into PudgyMon, and a ready prompt.

After generation → import → place (see [STUDIO_ASSETS.md](STUDIO_ASSETS.md)). Stand-in map: [ASSET_WISHLIST.md](ASSET_WISHLIST.md).

---

## Priority — Pudgy + Nest

### `char_pudgy_base_01` · target height **1.2**

**Plugs into:** `data/player_defaults.json` / `PlayerVisualSpec.model_id`

```
Cute chunky cartoon monster character for a party game called PudgyMon.
Round soft body like a dumpling with an oversized round head, stubby limbs,
big friendly eyes, tiny snout, rubbery plastic toy materials, coral-peach base color.
Standing idle A-pose, floor-pivoted, single character, no weapons, no text, family-friendly.
Clean PBR, exaggerated silhouette, game-ready low-to-mid poly.
```

### `env_nest_egg_01` · target height **2.0**

**Plugs into:** Nest centerpiece (replace greybox sphere)

```
Giant decorative party egg sculpture for a cute monster social hub.
Soft speckled shell, warm pastel orange and cream, rounded cartoon prop,
floor-pivoted, single object, no cracks with creatures emerging, no text.
```

### `prop_vibe_mushroom_01` · target height **1.8**

**Plugs into:** Nest flora décor

```
Oversized cartoon mushroom prop with glowing cap for a colorful party playground.
Thick stem, wide soft cap in coral or teal, slightly emissive toy plastic look,
floor-pivoted, single object, no characters.
```

---

## Global style (prepend to every prompt)

Use this block (or Studio’s style preset) on **every** job so the set matches:

```
Cartoon stylized 3D game asset for PudgyMon: Party Saga — cute chunky monster party world.
Bright readable colors, soft rounded edges, slightly rubbery plastic materials,
exaggerated silhouettes, clean PBR, no gore, no realistic dirt, no photorealism.
Single isolated object, centered, floor-pivoted (origin at ground center),
game-ready low-to-mid poly, no base/plinth, no floating text UI.
```

**Negative / avoid (if Studio supports it):**

```
photorealistic, grimdark, horror, blood, weapons, complex machinery internals,
tiny unreadable labels, multiple objects, diorama, landscape, character holding prop
```

**Export settings**

| Setting | Value |
|---------|--------|
| Format | GLB with baked Tripo PBR |
| Pivot | Floor center |
| Units | 1 unit ≈ 1 meter |
| Naming | Folder + file = `asset_id` / `asset_id.glb` |

---

## Batch A — High priority (playable replacements)

Generate these first. Rooms already have markers waiting.

### 1. Sort chute (reusable) — `env_sort_chute_01`

| Field | Value |
|-------|--------|
| **asset_id** | `env_sort_chute_01` |
| **target_height** | `1.5` |
| **Place on** | `sort_chute_*` in `data/rooms/hr_orientation.json` (reuse ×4; tint via materials later or generate color variants) |
| **Replaces** | `prop_pneumatic_tube_intake_funnel` |

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, slightly rubbery plastic materials,
exaggerated silhouettes, clean PBR, no gore, no realistic dirt, no photorealism.
Single isolated object, centered, floor-pivoted (origin at ground center),
game-ready low-to-mid poly, no base/plinth, no floating text UI, no people.

A freestanding pneumatic freight sort chute for a corporate space station:
wide funnel mouth on top, short vertical tube, open dump hatch at the bottom,
chunky hazard stripes, big cartoon intake rim, friendly industrial look,
orange and cream plastic with yellow caution bands, about 1.5 meters tall.
```

**Optional color variants** (same mesh, different prompt color line):

| asset_id | Accent color | Chute label (in-game) |
|----------|--------------|------------------------|
| `env_sort_chute_hot_dogs_01` | hot-dog orange / mustard | Hot Dogs |
| `env_sort_chute_toasters_01` | silver / chrome plastic | Toasters |
| `env_sort_chute_premium_air_01` | sky blue / white | Premium Air |
| `env_sort_chute_writeups_01` | angry red / pink | Write-Ups |

---

### 2. Shuttle seal door — `env_shuttle_seal_door_01`

| Field | Value |
|-------|--------|
| **asset_id** | `env_shuttle_seal_door_01` |
| **target_height** | `2.2` |
| **Place on** | `meltdown_door_left`, `meltdown_door_right` in `shuttle_meltdown.json` |
| **Replaces** | `env_break_glass_panel_01` |

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, slightly rubbery plastic materials,
exaggerated silhouettes, clean PBR, no gore, no realistic dirt, no photorealism.
Single isolated object, centered, floor-pivoted (origin at ground center),
game-ready low-to-mid poly, no base/plinth, no floating text UI, no people.

A freestanding cartoon airlock seal door panel for a shuttle bay:
tall rectangular bulkhead door with a big round viewport window,
chunky red seal lever, yellow-black hazard stripes, glowing green LOCKED light,
thick rubber gasket frame, about 2.2 meters tall, readable from a distance.
```

---

### 3. Crew character base — `char_crew_base_01`

| Field | Value |
|-------|--------|
| **asset_id** | `char_crew_base_01` |
| **target_height** | `1.6` |
| **Place via** | `PlayerVisualSpec.model_id` (see [CHARACTERS.md](CHARACTERS.md)) |
| **Replaces** | Capsule placeholder |

**Prompt:**

```
Cartoon stylized 3D game character for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, slightly rubbery materials,
exaggerated silhouettes, clean PBR, no gore, no photorealism.
Single isolated character, T-pose or neutral A-pose, floor-pivoted,
game-ready low-to-mid poly, no weapons, no base.

A stubby cartoon space freight contractor: big head, short legs, oversized gloves,
soft rubber limbs, simple jumpsuit with reflective stripes, blank friendly face,
neutral crew colors (yellow and navy accents), about 1.6 meters tall,
designed as a base mesh for hat and palette swaps, clear silhouette from third-person camera.
```

**Notes for Studio:** Prefer a simple single-mesh or few-part character. Hats are separate jobs (Batch C).

---

## Batch B — Medium priority (arena & signage)

### 4. Freight deck floor panel — `env_freight_deck_panel_01`

| Field | Value |
|-------|--------|
| **asset_id** | `env_freight_deck_panel_01` |
| **target_height** | `0.15` (thin) / or use `target_width: 4.0` |
| **Place on** | `floor_main` in `data/rooms/arena.json` (tile / repeat visually) |
| **Replaces** | Greybox slab |

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated object, centered, floor-pivoted, game-ready low-to-mid poly.

A square modular space-station freight deck floor panel tile:
thick metal plate with soft cartoon bevels, subtle rivets, painted lane stripes,
cool grey-blue with yellow safety chevrons, about 4 meters across and very thin,
designed to tile as a floor, no walls attached.
```

---

### 5. Room signs (per vault)

Generate three (or one reusable + recolor). Place on `room_sign` markers.

#### `env_room_sign_hr_01`

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated object, centered, floor-pivoted or freestanding post, game-ready.

A big cartoon corporate welcome sign on a short stand for an HR orientation bay:
wide horizontal board, bright yellow and blue, friendly but bureaucratic,
large blank face for baked text area, soft plastic look, about 3 meters wide and 1.5 meters tall including post.
```

#### `env_room_sign_breaker_01`

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated object, centered, freestanding, game-ready.

A big cartoon industrial warning sign for a breaker room:
wide board with electric bolt icon silhouette, blue and white corporate hazard style,
chunky frame, about 3 meters wide, readable silhouette, blank text panel area.
```

#### `env_room_sign_meltdown_01`

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated object, centered, freestanding, game-ready.

A big cartoon emergency shuttle bay sign:
wide board with red-orange meltdown / heat warning vibe, soft plastic materials,
flame or thermometer icon silhouette, chunky frame, about 3 meters wide, blank text panel area.
```

*(Cargo already has `cargo_ring_sign_01` — skip unless regenerating.)*

---

### 6. Meltdown floor glow disc — `vfx_meltdown_floor_glow_01`

| Field | Value |
|-------|--------|
| **asset_id** | `vfx_meltdown_floor_glow_01` |
| **target_height** | `0.05` |
| **target_width** | `8.0` |
| **Place on** | `meltdown_glow` in `shuttle_meltdown.json` |

**Prompt:**

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Single isolated flat disc, floor-pivoted, very thin, game-ready.

A large flat circular warning glow plate for a shuttle bay floor:
soft red-orange emissive cartoon hazard pattern, concentric rings,
no thickness, no walls, looks like a glowing safety decal about 8 meters wide.
```

---

## Batch C — Character hats (after base crew)

One hat mesh per roster slot. Parent under `char_crew_base_01` later. Keep pivots at head attachment (document in notes: “hat attach at origin”).

| # | asset_id | Prompt focus |
|---|----------|--------------|
| 1 | `char_hat_zip_01` | Lightning-bolt shaped soft cap, yellow/navy |
| 2 | `char_hat_grom_01` | Square cartoon hard hat, orange/brown |
| 3 | `char_hat_bloop_01` | Clear fishbowl helmet dome, cyan rim |
| 4 | `char_hat_taffy_01` | Stretchy neck scarf loop, pink/magenta (worn as head wrap) |
| 5 | `char_hat_rivet_01` | Spiky wrench-shaped hairpiece, grey/red |
| 6 | `char_hat_nix_01` | Hood with antenna stub, purple/black |
| 7 | `char_hat_boingo_01` | Two puffball antennae, lime/teal |
| 8 | `char_hat_pax_01` | Soft beret with tiny clipboard badge, beige/green |

**Shared hat prompt wrapper:**

```
Cartoon stylized 3D game accessory for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated hat/accessory only, no head, no body, centered at wear origin,
game-ready low poly, exaggerated silhouette readable from third-person camera.

[INSERT HAT DESCRIPTION FROM TABLE]
```

---

## Batch D — Low priority (scale-up content)

### Extra breaker panel — `env_breaker_panel_wall_01`

*(You already have `env_breaker_panel_01`; this is a wider wall-mounted bank for the GDD’s 12-switch layout.)*

```
Cartoon stylized 3D game prop for a slapstick space-freight comedy game.
Bright readable colors, soft rounded edges, clean PBR, no photorealism.
Single isolated object, floor-pivoted, game-ready.

A wide freestanding cartoon breaker panel bank with a row of chunky colorful toggle switches,
blue corporate housing, glowing status lights, soft plastic look, about 1.2 meters tall and 2 meters wide,
readable switch silhouettes, no tiny text.
```

### Extra coolant valve wheel — reuse / variant

You already have `prop_coolant_pipe_wheel_01/02`. For more valves, regenerate with:

```
... A large cartoon industrial coolant pipe valve handwheel on a short vertical stem,
bright red wheel, cyan pipe stub, soft rubber grips, about 0.8 meters tall ...
```

Suggested ids: `prop_coolant_pipe_wheel_03` … `_06`.

---

## Batch E — Nice-to-have props (fill dead space)

| asset_id | target_height | One-line prompt seed |
|----------|---------------|----------------------|
| `prop_hr_clipboard_stack_01` | 0.4 | Stack of oversized cartoon clipboards with sticky notes |
| `prop_explosive_egg_crate_01` | 1.0 | Freight crate stamped with cartoon egg + boom icon |
| `prop_writeup_form_kiosk_01` | 1.5 | Skinny HR kiosk that prints write-up tickets |
| `env_conveyor_short_01` | 0.8 | Short cartoon conveyor belt segment with rollers |
| `prop_ping_beacon_01` | 0.5 | Soft glowing teammate ping puck |
| `prop_dunce_hat_intern_01` | 0.5 | Tall dotted dunce cone for eliminated “intern” gag |

Use the **global style** block + the one-line seed as the full prompt.

---

## Suggested Studio job order

1. `env_sort_chute_01` (or 4 color variants)
2. `env_shuttle_seal_door_01`
3. `char_crew_base_01`
4. `env_freight_deck_panel_01`
5. Room signs (HR / Breaker / Meltdown)
6. `vfx_meltdown_floor_glow_01`
7. Hats (Batch C)
8. Scale-up / nice-to-haves

---

## After each pack

```bash
python scripts/import_immersive_studio_pack.py path/to/pack.zip
python scripts/validate_studio_assets.py
# Swap asset_id on the matching marker in data/rooms/*.json
cargo run -- local
```

If scale looks wrong: set `"uniform_scale"` in `studio_registry.json` or `"scale"` on the room marker — no re-export needed.
