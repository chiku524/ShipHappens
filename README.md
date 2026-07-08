# ShipHappens

**Competitive co-op escape room tournaments** in cartoon space-freight vaults. Cooperate to clear each stage, compete to survive the bracket — optional wager prize pools for top 3.

Built with **Bevy 0.19 (Rust)** · Third-person · Online multiplayer · Steam-bound indie

## Elevator pitch

Squid Game meets escape room: break into sealed corporate vault bays, survive absurd HR-themed challenges, and place top 3 in a 30-minute tournament. Solo or squad up — cartoon physics, snarky announcer, real stakes (practice first, wager later).

## Status

**Phase 1 — Tournament core (in design)**

- [x] Bevy engine + LAN multiplayer foundation
- [x] GLB asset pipeline + third-person camera
- [x] Full design docs for Vault Break pivot
- [ ] Tournament state machine + 4 MVP rooms
- [ ] Server-side scoring + solo 16 bracket
- [ ] Practice currency tournaments

See [docs/ROADMAP.md](docs/ROADMAP.md) for milestones.

## Requirements

- [Rust](https://rustup.rs/) 1.95+ (see `rust-toolchain.toml`)
- Windows PC (primary target; Linux/macOS for dev/CI)

## Quick start (dev prototype)

```bash
git clone https://github.com/chiku524/ShipHappens.git
cd ShipHappens

cargo run -- local          # offline greybox
cargo run -- host --port 7777
cargo run -- join --address 127.0.0.1 --port 7777
```

> Current build is a **networking/physics prototype** (crane + breakers greybox). Tournament rooms are not implemented yet.

## Documentation

| Doc | Description |
|-----|-------------|
| [GDD](docs/GDD.md) | Game vision & pillars |
| [TOURNAMENT](docs/TOURNAMENT.md) | 30-min brackets, elimination, Strikes |
| [SCORING](docs/SCORING.md) | Contribution Index & point tables |
| [ROOMS](docs/ROOMS.md) | Vault stage designs & scaling |
| [WAGERING](docs/WAGERING.md) | Prize pools, limits, compliance |
| [ROADMAP](docs/ROADMAP.md) | Development phases |
| [TECH](docs/TECH.md) | Bevy architecture |
| [CHARACTERS](docs/CHARACTERS.md) | Crew roster |
| [STEAM](docs/STEAM.md) | Store page draft |
| [STUDIO_ASSETS](docs/STUDIO_ASSETS.md) | 3D asset import workflow |
| [legacy/](docs/legacy/) | Archived v0.2 stowaway design |

## License

All rights reserved (solo indie — license TBD before public release).
