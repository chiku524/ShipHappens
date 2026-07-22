# Development Roadmap

> **Pivot:** Active product is **PudgyMon: Party Saga** (Nest + mini-games + Boing) — see [PARTY_ROADMAP.md](PARTY_ROADMAP.md) and [BRAND.md](BRAND.md). Vault tournament docs live under [archive/vault/](archive/vault/).

For the archived vault playable checklist, see [archive/vault/PLAYABLE_ROADMAP.md](archive/vault/PLAYABLE_ROADMAP.md).

## Phase 0 — Engine foundation ✓

- [x] Bevy 0.19 project at repo root
- [x] LAN host/join (bevy_replicon + renet)
- [x] GLB asset pipeline (Immersive Studio)
- [x] Third-person camera + server-authoritative interact
- [x] Headless multiplayer smoke test + CI

## Phase 1 — Tournament core (MVP) ← **IN PROGRESS**

**Goal:** Fun 30-minute solo bracket with practice currency. No real money.

- [x] Tournament state machine (`TournamentDirector`: lobby → rooms → elimination → finale → podium)
- [x] Room 1: HR Orientation Bay (`RoomRuntime` + vault objective interact)
- [x] Room 2: Cargo Ring Gantry (shared runtime + legacy crane interact)
- [x] Room 3: Breaker Panic (shared runtime + legacy breakers)
- [x] Finale: Shuttle Bay Meltdown (meltdown meter + vault objectives)
- [x] Server-side scoring (`ScoringService` + `scoring/ci.rs` point tables)
- [x] Solo dev bracket (4 slots, fast timers — scale to 16 for production)
- [x] Practice currency + payout UI (`PracticeLedger`)
- [x] Treasury Ghost announcer (`AnnouncerQueue` + HUD)

**Remaining for exit criteria:**
- [ ] Scale to solo 16 online with dedicated server
- [ ] Per-room geometry swap (distinct layouts vs shared greybox)
- [ ] Full 30-minute production timers (currently fast dev timers)

## Phase 2 — Team modes & Strikes

- [x] Duo / Trio / Squad slot scaling (`scaled_target`, `SlotSize`)
- [x] Contribution Index + Strike system
- [x] Leaseholder mechanic (`Leaseholder` component + `assign_leaseholder`)
- [x] Team composite scoring
- [ ] Premade party queue (network spawn only; UI pending)

**Exit criteria:** Squad 8 practice tournament with Strikes working.

## Phase 3 — Polish & retention

- [ ] Character models + slapstick animations
- [ ] Audio pass (SFX, music, full PA library)
- [x] Room Mastery badges scaffold (`RoomMastery`)
- [x] Seasonal Vault Set registry stub (`SeasonalVaultSet`)
- [x] Spectator component stub
- [ ] Steam lobby integration (`SteamLobbyConfig` stub only)

**Exit criteria:** Steam playtest build (practice only).

## Phase 4 — Wager infrastructure (gated)

> Requires legal review before implementation.

- [x] Practice rank + queue gates scaffold (`WagerGate`, `Wallet`)
- [x] Wallet, deposit caps, loss limits (data model)
- [ ] Age verification + geo-restrictions (integration)
- [x] Payout pipeline math (`PayoutCalculator`, 50/30/20)
- [x] Audit log stub (`AuditLog`)
- [ ] Responsible gaming UI

**Exit criteria:** Wager mode live in allowed jurisdictions only.

## Phase 5 — Live ops

- [x] Leaderboard stub (`Leaderboard`)
- [x] Handshake side bets scaffold (`SideBetBoard`)
- [x] King of the Vault mode stub (`KingOfTheVaultState`)
- [ ] Double-or-nothing side rooms
- [x] Remnant clue board stub (`RemnantClueBoard`)

---

## Code map (implemented)

| Module | Path | Phase |
|--------|------|-------|
| Tournament director | `src/tournament/` | 1–2 |
| Scoring / CI | `src/scoring/` | 1–2 |
| Room runtime | `src/rooms/` | 1–2 |
| Economy / wallet | `src/economy/` | 1, 4 |
| Announcer | `src/announcer/` | 1 |
| Meta / mastery | `src/meta/` | 3 |
| Live ops | `src/live_ops/` | 5 |

## Design reference

| Doc | Contents |
|-----|----------|
| [GDD.md](GDD.md) | Game vision |
| [TOURNAMENT.md](TOURNAMENT.md) | Brackets & timing |
| [SCORING.md](SCORING.md) | Points & CI |
| [ROOMS.md](ROOMS.md) | Vault stages |
| [WAGERING.md](WAGERING.md) | Economy |
| [legacy/](legacy/) | v0.2 stowaway design (archived) |

## Run locally

```bash
cargo run -- local
```

Press **F** at the green vault pad during active rooms. Tournament auto-advances with 3 bots on fast timers (~2 min full run).
