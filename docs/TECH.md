# PudgyMon — Technical Design

## Stack

| Layer | Choice |
|-------|--------|
| Engine | Bevy 0.19 |
| Language | Rust |
| Networking | bevy_replicon + bevy_replicon_renet (ENet-style LAN) |
| Voice (later) | Steam Voice |
| Version control | Git + GitHub |
| Assets | Blender + Immersive Studio / Tripo + kitbash (GLB) |
| Target | Windows PC, Steam |

## Why Bevy

- Rust performance and type safety for a networked co-op game
- ECS architecture fits job stations, replication, and round state
- bevy_replicon provides server-authoritative replication patterns
- Cross-platform dev with a single codebase

## Architecture (current prototype)

```
Host (authoritative server)
 ├── JobSystem        — legacy greybox job validation (to be replaced)
 ├── NetworkPlugin    — renet transport, player spawn on connect
 ├── InteractionPlugin — InteractRequest client → server RPC
 └── SmokeAutomation  — headless CI checks

Client
 ├── LocalPlayer      — assigned on connect (online) or at spawn (offline)
 ├── MoveInput        — camera-relative WASD sent to server when online
 ├── ThirdPersonCamera — mouse orbit, scroll zoom
 └── UiPlugin         — minimal debug HUD
```

## Planned tournament architecture (Phase 1)

See [GDD.md](GDD.md), [TOURNAMENT.md](TOURNAMENT.md), [SCORING.md](SCORING.md).

```
Dedicated server / listen host
 ├── TournamentDirector  — lobby → rooms → elimination → finale → podium
 ├── RoomRuntime         — per-stage objectives, timers, scaling by slot size
 ├── ScoringService      — server-side CI + composite (all raw events logged)
 ├── SlotRegistry        — solo / duo / trio / squad slots
 └── WagerLedger         — practice currency now; real wallet gated (Phase 4)

Replicated state
 ├── TournamentPhase, RoomId, TimeRemaining
 ├── SlotCompositeScores, PlayerCI
 ├── StrikeCount (team modes)
 └── EliminationOrder
```

**Server validates:** all objective progress, scoring, elimination, buy-in lock.
**Clients send:** movement + interaction intent only.

## Networking rules

**Server validates:**
- Job progress (`JobSystem::handle_interact`)
- Player movement (`apply_move_input` from `MoveInput` events)

**Replicated:**
- `JobBoard`, `NetworkPlayer`, `PlayerName`, `PlayerColor`
- `SmokeJobFlags` (CI smoke test helper)

**Client sends:**
- `MoveInput { direction, sprint }` each frame when moving
- `InteractRequest { station: Entity }` on F press

Default port: **7777**

## Data files

| File | Purpose |
|------|---------|
| `data/job_manifest.json` | All 10 job definitions (id, zone, target, satisfaction) |
| `assets/studio_registry.json` | Immersive Studio asset IDs → GLB paths |

GLBs load from `assets/models/{asset_id}/{asset_id}.glb` via Bevy `AssetServer`.

## Binaries

| Binary | Purpose |
|--------|---------|
| `pudgymon` | Interactive game (`cargo run` / `host` / `join`) |
| `pudgymon_smoke` | Headless LAN smoke test for CI |

## CI

`.github/workflows/multiplayer-smoke.yml` runs `cargo test` and `scripts/run_mp_smoke_test.sh`.

## Migration notes

- **Engine:** Migrated from Godot 4.7 to Bevy 0.19 (2026).
- **Design:** Pivoted from Crew vs Stowaway co-op to **Vault Break** tournament escape rooms. See [GDD.md](GDD.md); legacy design in [legacy/](legacy/).
