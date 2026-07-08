# ShipHappens

**6–8 player cartoon space freight co-op** with social deduction. Friends complete absurd jobs on a discount orbital station while one or two **Stowaways** smuggle contraband and sabotage the run.

Built with **Bevy 0.19 (Rust)** · Third-person · LAN multiplayer · Steam-bound indie

## Elevator pitch

You and your friends are the worst freight crew in the galaxy. Load weird **space goods**, survive slapstick physics, and catch the Stowaway before ShipHappens Logistics fires you all.

## Status

**Bevy migration — early vertical slice**

- [x] Job manifest + Immersive Studio asset registry loading
- [x] LAN host/join (port 7777) via bevy_replicon + renet
- [x] Crane of Regret + Power Hour jobs (greybox level)
- [x] Third-person orbit camera + server-authoritative interact
- [x] Headless multiplayer smoke test for CI
- [ ] Remaining 8 jobs, full station map, stowaway/sabotage, meetings

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full plan.

## Requirements

- [Rust](https://rustup.rs/) 1.95+ (see `rust-toolchain.toml`)
- Windows PC (primary target; Linux/macOS for dev/CI)

## Quick start

```bash
git clone https://github.com/chiku524/ShipHappens.git
cd ShipHappens

# Offline greybox
cargo run -- local

# LAN (two terminals)
cargo run -- host --port 7777
cargo run -- join --address 127.0.0.1 --port 7777
```

Walk **north** to the crane console or **east** to the breaker panels. Press **F** to interact.

## Controls

| Key | Action |
|-----|--------|
| WASD | Move (relative to camera) |
| Mouse | Orbit camera |
| Shift | Sprint |
| F | Interact with nearest station |
| Scroll | Zoom camera |
| Esc | Release / capture mouse |

## Tests

```bash
cargo test                              # unit + integration
bash scripts/run_mp_smoke_test.sh       # headless 2-player smoke
```

## Documentation

| Doc | Description |
|-----|-------------|
| [GDD](docs/GDD.md) | Full game design document |
| [ROADMAP](docs/ROADMAP.md) | Solo dev milestones |
| [TECH](docs/TECH.md) | Engine, networking, architecture |
| [CHARACTERS](docs/CHARACTERS.md) | Eight default crew roster |
| [JOBS](docs/JOBS.md) | All 10 station jobs |
| [STOWAWAY](docs/STOWAWAY.md) | Smuggle routes and sabotage |
| [STEAM](docs/STEAM.md) | Store page draft and tags |
| [STUDIO_ASSETS](docs/STUDIO_ASSETS.md) | Immersive Studio → Tripo → GLB import workflow |

## Immersive Studio assets (Tripo → GLB)

3D props and environment pieces are generated with **Immersive Labs Studio** (Tripo mesh + PBR) and imported via:

```bash
python scripts/import_immersive_studio_pack.py path/to/pack.zip
```

See [docs/STUDIO_ASSETS.md](docs/STUDIO_ASSETS.md) for the full workflow.

## Project structure

```
ShipHappens/
├── src/              # Bevy game (app, jobs, network, player, world, …)
├── tests/            # Integration tests
├── scripts/          # Asset import + multiplayer smoke test
├── assets/           # GLBs, textures, studio_registry.json
├── data/             # job_manifest.json
├── docs/             # Design & planning markdown
├── Cargo.toml
└── rust-toolchain.toml
```

## License

All rights reserved (solo indie — license TBD before public release).
