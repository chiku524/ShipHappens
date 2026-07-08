# ShipHappens — Game Design Document (v1.0)

## Summary

| Field | Value |
|-------|--------|
| **Title** | ShipHappens: Vault Break |
| **Genre** | Competitive co-op escape rooms / party tournament |
| **Players** | 16–32 per bracket (solo, duo, trio, or squad slots) |
| **Session** | ~30 minute tournament |
| **Camera** | Third-person orbit |
| **Tone** | Full cartoon — ragdoll, slapstick, no gore |
| **Platform** | Windows PC (Steam first) |
| **Monetization** | Cosmetics + optional wager tournaments (see [WAGERING.md](WAGERING.md)) |

**Elevator pitch:** Squid Game meets escape room — cooperate to clear each vault, compete to survive the bracket. Cartoon freight contractors break into decommissioned corporate vaults while an snarky HR AI narrates your inevitable write-up.

## Design pillars

1. **Cooperate under pressure** — rooms require teamwork; brackets require beating others
2. **Readable chaos** — bright colors, big silhouettes, clear UI at 16+ players
3. **Tournament arc** — 30-minute rise-and-fall stories every match
4. **Fair stakes** — skill-based scoring before real-money queues unlock
5. **Clip-friendly** — third-person slapstick, announcer barks, podium moments

## Fantasy

**You are:** Contractors for ShipHappens Logistics breaking into sealed vault bays on derelict stations.

**You do:** Clear absurd escape-room challenges (sort freight, run gantries, flip breakers, survive meltdowns).

**You win:** Advance through the bracket, place top 3, split the prize pool.

**Tone:** Full cartoon. "Death" = bonked out → brief ragdoll → respawn as intern with dunce prop.

## Core loop

```
Lobby (buy-in lock) → Room 1 → Elimination
                   → Room 2 → Elimination (+ Strikes)
                   → Room 3 → Elimination (+ Strikes)
                   → Finale → Podium + payout
```

See [TOURNAMENT.md](TOURNAMENT.md) for bracket sizes, timing, and elimination rules.
See [ROOMS.md](ROOMS.md) for room archetypes and MVP stage list.
See [SCORING.md](SCORING.md) for Contribution Index and point values.

## Player modes

| Mode | Slot size | Elimination unit | Primary audience |
|------|-----------|------------------|------------------|
| **Solo Vault** | 1 | Whole slot | Randos, wager players without a squad |
| **Duo Breach** | 2 | Whole team (+ Strikes) | Pairs |
| **Trio Shift** | 3 | Whole team (+ Strikes) | Small friend groups |
| **Squad Contract** | 4 | Whole team (+ Strikes) | Full co-op squads |

Brackets are **homogeneous** — all solo or all squad in a given tournament, never mixed.

## Variable team size

Team size is chosen at queue time. Rooms use the **same blueprint** with scaled objectives (see [ROOMS.md](ROOMS.md#room-scaling)).

## Scoring & elimination philosophy

- **Team/solo composite score** determines who enters the danger zone each round
- **Contribution Index (CI)** tracks individual performance within a team
- **Strikes** punish lowest-CI players on surviving teams that landed in the danger zone
- **2 Strikes = eliminated** even if the team advances

Solo tournaments skip Strikes — whole slot elimination only.

## Wager tournaments (high level)

- Account wallet with deposit caps and weekly limits
- Buy-in tiers ($1 / $5 / $10)
- **5%** platform rake → treasury operations
- **95%** → prize pool; **top 3** split 50% / 30% / 20%

Full economy, compliance notes, and anti-abuse rules: [WAGERING.md](WAGERING.md).

> **Legal note:** Real-money wagering requires jurisdiction-specific licensing, age verification, and geo-restrictions. ShipHappens should launch with **practice currency** first; wager mode is a gated later phase.

## Signature mechanics

| Mechanic | Description |
|----------|-------------|
| **The Lease** | Rotating "Leaseholder" sees full objectives; others see fragments. Leaseholder directs but cannot interact. |
| **Treasury Ghost** | Snarky HR announcer AI; narrates eliminations and fake "corporate shortcuts" (traps). |
| **Double-or-nothing door** | Optional harder side room: skip ahead one seed or land at bottom of elimination pile. |
| **Remnant clues** | Eliminated slots leave a holographic hint in the next room (slight leader slowdown). |
| **Handshake wagers** | Two slots side-bet a % of their own payout on head-to-head placement next room. |

## Map theme: Vault Break

Reuse the ShipHappens space-freight aesthetic. Each tournament room is a sealed bay on a decommissioned orbital station:

```
[ SHUTTLE BAY ]     ← Finale: Meltdown Vault
      |
[ CARGO RING ]      ← Room 2: Gantry
      |
[ BREAK ROOM ]      ← Meeting / spectate
      |
[ MAIN HUB ]        ← Room 1: HR Orientation
      |
[ OPS DECK ]        ← Room 3: Breaker Panic
```

Seasonal reskins (haunted mansion, museum heist) reuse mechanics with new art packs.

## Meta progression (non-P2W)

- **Room Mastery badges** — clear a room type N times → cosmetic title
- **Tool cosmetics** — grappling hook skins, ping flairs (no wager-queue power)
- **Seasonal Vault Sets** — 4–6 new rooms per season
- **Crew Reputation** — repeat teammates get faster revives (co-op QoL, not score bonus)

## Related documents

| Doc | Contents |
|-----|----------|
| [TOURNAMENT.md](TOURNAMENT.md) | 30-min timeline, brackets, elimination, Strikes |
| [SCORING.md](SCORING.md) | CI formula, point tables, normalization |
| [ROOMS.md](ROOMS.md) | Room archetypes, MVP stages, scaling |
| [WAGERING.md](WAGERING.md) | Economy, payouts, limits, anti-abuse |
| [ROADMAP.md](ROADMAP.md) | Development phases |
| [legacy/](legacy/) | Superseded v0.2 stowaway design |

## Superseded design (v0.2)

The original **Crew vs Stowaway** social-deduction freight co-op is archived in [legacy/](legacy/). Art assets and cartoon tone carry forward; core loop is replaced by tournament escape rooms.
