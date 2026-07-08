# Development Roadmap

## Phase 0 — Engine foundation ✓

- [x] Bevy 0.19 project at repo root
- [x] LAN host/join (bevy_replicon + renet)
- [x] GLB asset pipeline (Immersive Studio)
- [x] Third-person camera + server-authoritative interact
- [x] Headless multiplayer smoke test + CI

## Phase 1 — Tournament core (MVP) ← **NEXT**

**Goal:** Fun 30-minute solo bracket with practice currency. No real money.

- [ ] Tournament state machine (lobby → rooms → elimination → finale → podium)
- [ ] Room 1: HR Orientation Bay (sort mini-game)
- [ ] Room 2: Cargo Ring Gantry (physics co-op)
- [ ] Room 3: Breaker Panic (asymmetric info)
- [ ] Finale: Shuttle Bay Meltdown
- [ ] Server-side scoring ([SCORING.md](SCORING.md))
- [ ] Solo 16 bracket ([TOURNAMENT.md](TOURNAMENT.md))
- [ ] Practice currency + fake payout UI
- [ ] Treasury Ghost announcer (text + placeholder VO)

**Exit criteria:** 16-player solo practice tournament playable end-to-end online.

## Phase 2 — Team modes & Strikes

- [ ] Duo / Trio / Squad slot scaling per [ROOMS.md](ROOMS.md#room-scaling)
- [ ] Contribution Index + Strike system
- [ ] Leaseholder mechanic
- [ ] Team composite scoring
- [ ] Premade party queue

**Exit criteria:** Squad 8 practice tournament with Strikes working.

## Phase 3 — Polish & retention

- [ ] Character models + slapstick animations
- [ ] Audio pass (SFX, music, full PA library)
- [ ] Room Mastery badges + cosmetics
- [ ] Seasonal Vault Set #1 (2 additional rooms)
- [ ] Spectator mode
- [ ] Steam lobby integration

**Exit criteria:** Steam playtest build (practice only).

## Phase 4 — Wager infrastructure (gated)

> Requires legal review before implementation.

- [ ] Practice rank + queue gates ([WAGERING.md](WAGERING.md))
- [ ] Wallet, deposit caps, loss limits
- [ ] Age verification + geo-restrictions
- [ ] Payout pipeline (top 3, 50/30/20)
- [ ] Audit log + dispute replay
- [ ] Responsible gaming UI

**Exit criteria:** Wager mode live in allowed jurisdictions only.

## Phase 5 — Live ops

- [ ] Seasonal brackets + leaderboards
- [ ] Handshake side bets
- [ ] King of the Vault mode
- [ ] Double-or-nothing side rooms
- [ ] Remnant clue system

---

## Design reference

| Doc | Contents |
|-----|----------|
| [GDD.md](GDD.md) | Game vision |
| [TOURNAMENT.md](TOURNAMENT.md) | Brackets & timing |
| [SCORING.md](SCORING.md) | Points & CI |
| [ROOMS.md](ROOMS.md) | Vault stages |
| [WAGERING.md](WAGERING.md) | Economy |
| [legacy/](legacy/) | v0.2 stowaway design (archived) |

## Superseded work

Godot vertical slice and Crew vs Stowaway loop are **archived**. Existing greybox code (crane, breakers) remains as prototyping scaffolding until tournament rooms replace it.
