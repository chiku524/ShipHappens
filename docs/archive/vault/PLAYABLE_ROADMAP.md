# Playable Product Roadmap

Checklist from “dev greybox” → “something people would actually play.”  
Check items off as they ship. Technical tournament scaffolding is largely done; this list is **content, feel, and presentation**.

Related: [ROADMAP.md](ROADMAP.md) (engine phases) · [DROP_IN.md](DROP_IN.md) · [PACKAGING.md](PACKAGING.md) · [STUDIO_PROMPTS.md](STUDIO_PROMPTS.md) · [ASSET_WISHLIST.md](ASSET_WISHLIST.md) · [GDD.md](GDD.md)

---

## How to use

1. Work **top-down within each phase** (P0 before P1).
2. Prefer finishing a **vertical slice** before scaling rooms/players.
3. Mark `[x]` when playable in a build friends can try — not when stubbed.

**Definition of done for this doc:** a stranger can install/run, understand the first room in ~30s, laugh at slapstick, finish a short tournament, and want a rematch — without reading the repo.

---

## Phase A — Vertical slice (HR Orientation)

*One finished room that looks and feels like the game.*

### Art & world
- [ ] Crew base character GLB (`char_crew_base_01`) replaces capsule
- [ ] Character idle + run + interact animations
- [ ] Dedicated sort chutes (4 variants or 1 reusable + tints) — see [STUDIO_PROMPTS.md](STUDIO_PROMPTS.md)
- [ ] Room-specific floor / walls (not only shared arena shell)
- [ ] HR room sign (dedicated or polished stand-in)
- [x] Lighting pass for HR bay (readable, cartoon, not flat greybox)

### Gameplay (real mini-game, not only “press F”)
- [x] Pick up freight item (carryable) before sorting
- [x] Drop / throw into correct chute under time pressure
- [x] Wrong chute feedback (zap / bounce / write-up bark)
- [x] Conveyor or spawn cadence so room stays busy
- [x] Teach loop in first 30s without reading docs (prompts / PA / arrows)

### Juice & audio
- [x] Interact SFX (pickup, sort correct, sort wrong)
- [ ] Short music bed for HR
- [ ] Announcer bark on room start / clear / wrong sort (temp VO ok)
- [x] Camera or character juice on success/fail (bob, shake, flash)

### UI (non-debug)
- [x] Objective card for HR (“Sort into: Hot Dogs”) readable as game UI
- [x] Progress / timer presentation that isn’t a debug dump
- [x] Hide or demote server/client debug lines in player-facing builds

**Exit:** Friends can play HR alone and say “I get it” without you explaining.

---

## Phase B — Look good (visual / audio bar)

*Readable fantasy across the product, not just one room.*

### Characters
- [ ] Ragdoll / bonk-out pose (even short)
- [ ] Intern / dunce respawn gag prop
- [ ] Eight hat meshes (Zip → Pax) — [CHARACTERS.md](CHARACTERS.md)
- [ ] Palette / material swaps per crew slot
- [ ] Emote set MVP (point, shrug, panic, thumbs up) — optional until party slice

### Rooms & props
- [ ] Distinct layout geometry per vault (HR / Gantry / Breaker / Meltdown)
- [ ] Shuttle seal doors replace breaker-panel stand-ins
- [ ] Freight deck floor panel / tiling for arena
- [x] Meltdown floor glow VFX asset
- [ ] Per-room signage (Breaker, Meltdown; Cargo sign exists)
- [ ] Prop scale / pivot pass so nothing floats or sinks

### Presentation
- [x] Global lighting / color grade direction locked (per-room key/fill)
- [x] Sparks, clear pulses, elimination VFX
- [ ] Full audio pass: music beds per phase, SFX library, PA bank
- [x] Main menu / title screen that sells the brand (not CLI)
- [x] Elimination screen with drama (danger zone, cuts, remnant hint)
- [x] Podium screen with placements + practice VC payout

**Exit:** Trailer-length footage doesn’t need “imagine it’s polished” narration.

---

## Phase C — Feel playable (game feel)

*Each room is a different verb; slapstick is real.*

### Shared feel
- [x] Ground collision / no clipping through floors
- [x] Soft collisions with props / walls (or push volumes)
- [x] Knockback / bonk on wrong breaker or hard hits
- [x] Carryables can be dropped / caught (physics comedy baseline)
- [x] Clear win/lose feedback beyond a HUD string
- [x] Human agency: bots don’t carry the run; skill decides placement

### Room 1 — HR Orientation Bay
- [x] Phase A complete (above)
- [x] Solo clear feels earned under pressure

### Room 2 — Cargo Ring Gantry
- [x] Aim / lift / catch roles (solo can do all)
- [x] Drop penalty + catch teammate drop fantasy
- [x] Crane juice (hook motion, thud, near-miss)

### Room 3 — Breaker Panic
- [x] Sequence puzzle with readable wrong-order zap
- [x] Info asymmetry hooks for duo+ (panel labels / pings)
- [x] Scale toward GDD switch count when party mode exists

### Finale — Shuttle Bay Meltdown
- [x] Shared meltdown meter readable as panic UI
- [x] Coolant / load / seal as distinct verbs
- [x] Fail state feels catastrophic (not quiet timer end)
- [x] Success → podium beat with announcer payoff

### Pacing
- [x] Production-length timers option for “real” matches
- [x] Fast timers remain for daily iteration
- [x] Full 4-room arc feels like a tournament story, not a slideshow

**Exit:** A full solo run is fun even with bots; you want a second run.

---

## Phase D — Party product (people choose it)

*Friends can play without Discord + cargo args.*

### UX / flow
- [x] Main menu: Local / Host / Join
- [x] Lobby with ready-up (no 5s auto-start for party)
- [x] Host can start when ready
- [x] In-game pause (Esc)
- [x] Leave / rematch flow
- [x] Settings: sensitivity, volume, fullscreen

### Multiplayer
- [ ] Reliable 2–4 player party on LAN
- [x] Correct local player ownership (no wrong capsule)
- [x] Host plays as listen-server (already started — keep solid)
- [x] Disconnect / reconnect or clean leave messaging
- [x] Rematch resets lobby cleanly (jobs, carry, human slots, spectate)
- [x] Spectate eliminated players (stub → real)

### Packaging
- [x] Windows build players can install/run without Rust toolchain
- [ ] Steam playtest page / depot (practice only)
- [x] Crash / log path for playtesters

### Scale (after party slice is fun)
- [x] Solo 16 online bracket
- [x] Dedicated server path
- [x] Duo / Trio / Squad queue UI
- [x] Leaseholder UX (pings, callouts) for teams

**Exit:** Four friends finish a practice tournament and ask for rematch without you hosting their machines manually.

---

## Phase E — Retention & live ops (after the loop is fun)

*Do not prioritize these before Phases A–D.*

- [x] Room Mastery badges (wired to real clears)
- [ ] Seasonal vault set rotation
- [ ] Cosmetics / hat unlocks
- [ ] Leaderboards (practice)
- [ ] Steam lobby / friends invite
- [ ] Handshake side bets / King of the Vault (design stubs exist)
- [ ] Wager mode — **only after legal review** ([WAGERING.md](WAGERING.md))

---

## Suggested work order (sprint sequence)

Use this when picking “what next”:

| Sprint | Focus | Done when |
|--------|-------|-----------|
| 1 | Phase A — HR vertical slice | Friends “get it” in HR alone |
| 2 | Phase B juice for that slice | Trailerable 30–60s of HR |
| 3 | Phase C — second room (Breaker or Gantry) | Two distinct verbs feel different |
| 4 | Phase D — menu + 4p party | Friends play without CLI |
| 5 | Remaining rooms + production pacing | Full tournament feels like a show |
| 6 | Steam playtest build | External playtesters install & finish a run |
| ∞ | Phase E | Only after rematch rate is good |

---

## Quick status snapshot

| Area | Today (honest) | Target |
|------|----------------|--------|
| Tournament state machine | Working | Keep |
| Asset drop-in pipeline | Working | Keep feeding [STUDIO_PROMPTS.md](STUDIO_PROMPTS.md) |
| Characters | Capsules | Rig + anims + hats |
| Rooms | Prop swaps in one shell | Distinct geometry + real verbs |
| UI | Menu + party options + act cards + podium | Art polish |
| Audio | Catalog + Pitch fallback | Music beds + VO files |
| Physics / slapstick | Soft push + knockback + sparks | Ragdoll / catch comedy |
| Party UX | Menu options + dedicated + team sizes | Real LAN soak |
| Steam | Stub + zip package path | Playtest depot |

---

## Checklist hygiene

- Prefer **closing Phase A** over scattering partial checks across B–E.
- If an item is “scaffolded in code but not player-facing,” leave it `[ ]`.
- When checking art: also check **in-game placement + scale**, not only registry import.
