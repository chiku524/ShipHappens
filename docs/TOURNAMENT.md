# Tournament Structure

## Overview

A standard ShipHappens tournament runs **~30 minutes**: four stages (three elimination rounds + finale), with ~2 minutes for lobby lock and podium payout.

Brackets are **homogeneous by slot size** — all solo, all duo, all trio, or all squad. Never mix slot sizes in one bracket.

## Bracket sizes

| Bracket | Starting slots | After R1 | After R2 | After R3 | Finale |
|---------|----------------|----------|----------|----------|--------|
| **Solo 16** | 16 | 12 | 8 | 4 | 3 paid |
| **Solo 32** | 32 | 24 | 16 | 8 | 3 paid |
| **Squad 8** | 8 teams | 6 | 4 | 2 (+ 2nd place race) | 3 teams paid* |
| **Squad 4** | 4 teams | 3 | 2 | — | 3 teams paid* |

\*Small squad brackets may use a **placement finale** where all survivors are ranked 1st–3rd by individual CI rather than eliminating to one winner.

**Default MVP target:** Solo 16.

## 30-minute timeline (Solo 16)

| Time | Phase | Duration |
|------|-------|----------|
| 0:00 | Lobby + buy-in lock | 2 min |
| 2:00 | **Room 1** — HR Orientation Bay | 5 min |
| 7:00 | Elimination + announcer bark | 1 min |
| 8:00 | **Room 2** — Cargo Ring Gantry | 6 min |
| 14:00 | Elimination + Strike assignment | 1 min |
| 15:00 | **Room 3** — Breaker Panic | 6 min |
| 21:00 | Elimination + Strike assignment | 1 min |
| 22:00 | **Finale** — Shuttle Bay Meltdown | 7 min |
| 29:00 | Podium + payout ceremony | 1 min |

Room timers are **hard caps**. Incomplete rooms resolve by lowest composite score among unfinished slots.

## Elimination rules

### Solo mode

Every elimination removes **whole slots** (the player).

| Round | Start → End | Cut |
|-------|-------------|-----|
| Room 1 | 16 → 12 | Bottom 4 composite scores |
| Room 2 | 12 → 8 | Bottom 4 |
| Room 3 | 8 → 4 | Bottom 4 |
| Finale | 4 → 3 paid | Rank by finale CI (room clears for all 4; payout by placement) |

No Strikes in solo mode.

### Team mode (duo / trio / squad)

**Primary:** Whole team eliminated when in the danger zone.

| Round | Teams cut | Strike behavior |
|-------|-----------|-----------------|
| Room 1 | Bottom 25% | No Strikes |
| Room 2 | Bottom 33% of remaining | Lowest CI on each **surviving** danger-zone team gets **1 Strike** |
| Room 3 | Bottom 50% of remaining | Lowest CI on each surviving danger-zone team gets **1 Strike** |
| Finale | — | 2 Strikes at any time = player eliminated before finale starts |

**Danger zone:** Teams whose composite score ranks in the elimination cut for that round.

**Surviving danger zone:** Teams that finished in the cut band but are saved by tie-breaker or sudden-death (see below) — rare; most danger-zone teams are eliminated outright.

#### Strike examples (Squad 8)

- Room 2: 8 → 6 teams cut (bottom 2 eliminated). If Team Alpha was 7th but saved by tie-break, lowest CI on Alpha gets a Strike.
- A player with 2 Strikes is **eliminated** before Room 3 regardless of team placement.

### Tie-breaking

When composite scores tie at the cut line:

1. **Sudden-death micro-game** (10 seconds): memory tile, button sequence, or balance beam
2. If still tied: higher average CI across prior rooms wins
3. If still tied: coin flip (logged for audit)

## Composite team/solo score (per room)

Used to determine danger zone and elimination order:

```
Composite = (Room Cleared ? 100 : 0) × 0.40
          + Speed Score × 0.25
          + Efficiency Score × 0.20
          + Cooperation Score × 0.15
```

- **Speed Score:** `100 × (fastest_clear_time / your_clear_time)`, capped at 100
- **Efficiency Score:** `100 − penalty_points` (see [SCORING.md](SCORING.md))
- **Cooperation Score:** normalized support points within slot

Incomplete rooms: `Room Cleared = 0`; speed/efficiency/cooperation still computed from partial progress.

## Slot sizes & matchmaking

| Slot | Players | Queue name |
|------|---------|------------|
| Solo | 1 | Solo Vault |
| Duo | 2 | Duo Breach |
| Trio | 3 | Trio Shift |
| Squad | 4 | Squad Contract |

Players pick slot size at queue. Wager lobbies additionally filter by **buy-in tier** ($1 / $5 / $10).

### Friends & fill

- **Premade:** Full slot queues together
- **Fill:** Open slots matchmade; solo wager is the default for players without a trusted team

## Modes beyond standard bracket

| Mode | Description |
|------|-------------|
| **Practice Vault** | Same rooms, fake currency, no rake |
| **Squad Rush** | Fixed premade squad, casual only |
| **Free Agent** | Matchmade team each tournament (future) |
| **King of the Vault** | Winners stay; challengers rotate (future) |
| **Spectator** | Watch wager finals; cosmetic cheer effects only |

## Announcer beats (Treasury Ghost)

| Event | Sample bark |
|-------|-------------|
| Room start | *"Welcome to Orientation Bay. Compliance is mandatory and fun."* |
| Danger zone | *"Slots 14–16, please report to the Voluntary Separation Airlock."* |
| Strike | *"Performance note added to your permanent record."* |
| Finale | *"Meltdown imminent. Heroism is voluntary; payouts are not."* |
| Podium | *"Congratulations. Your earnings are taxable in 47 sectors."* |

## Related

- [SCORING.md](SCORING.md) — CI formula and point tables
- [ROOMS.md](ROOMS.md) — per-room objectives
- [WAGERING.md](WAGERING.md) — buy-in and payout math
