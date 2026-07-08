# Scoring & Contribution Index

## Overview

ShipHappens uses two scoring layers:

1. **Composite Score** — per slot (solo player or team) per room; drives elimination
2. **Contribution Index (CI)** — per player per room; drives Strikes, finale payout splits, and MVP

All raw points are **normalized to 0–100 within the slot** each room before applying weights.

## Contribution Index (CI)

```
CI = (Objective Points × 0.45)
   + (Support Points × 0.25)
   + (Efficiency × 0.20)
   + (Clutch Bonus × 0.10)
```

Each component is normalized 0–100 within the slot after raw points are tallied.

### Normalization

```
normalized_component = (player_raw / slot_top_raw) × 100
```

If `slot_top_raw == 0`, component = 0 for all.

---

## Objective Points (raw)

Direct progress toward room completion.

| Action | Points | Notes |
|--------|--------|-------|
| Correct sort / placement | +8 | Per item (HR Orientation) |
| Incorrect sort | −3 | Per item |
| Crate delivered to zone | +15 | Gantry room |
| Breaker flipped correctly | +12 | Per switch (Breaker Panic) |
| Breaker flipped wrong | −8 | Per switch |
| Coolant valve turned | +10 | Finale |
| Escape crate loaded | +20 | Finale |
| Door sealed | +15 | Finale |
| Sub-task completed (parallel room) | +25 | One-time per sub-task |
| Room clear bonus | +50 | Split among active participants by % contribution |

### Participation floor

Players with **< 5%** of slot objective points in a room:
- Cannot receive MVP or finale bonus multipliers
- Auto-flagged for **lowest CI** consideration if team is in danger zone
- If idle > 8 seconds cumulative: −20 Efficiency (see below)

---

## Support Points (raw)

Actions that help teammates without direct objective credit.

| Action | Points | Notes |
|--------|--------|-------|
| Revive teammate | +20 | Per revive |
| Hold door / gate open (per 3s) | +5 | Cap +30 per room |
| Carry item for teammate (delivery) | +10 | Must result in teammate deposit |
| Correct ping on objective | +3 | Cap +15 per room |
| Leaseholder correct callout | +8 | Per validated callout (Lease mechanic) |
| Catch teammate's dropped object | +12 | Gantry / physics rooms |
| Buff shared tool (if applicable) | +6 | Per use |

---

## Efficiency (raw)

Starts at **100** per player per room; penalties subtract.

| Event | Penalty | Notes |
|-------|---------|-------|
| Fall / ragdoll bonk | −5 | Per incident |
| Dropped carryable | −8 | Per drop |
| Wrong input (non-breaker) | −4 | Per mistake |
| Idle > 8s (cumulative) | −20 | Once per room |
| Team damage (grief) | −50 | Once; flags review |
| Write-Up Meter tier crossed | −10 | Per tier (team rooms) |
| Sudden-death loss | −30 | Tie-breaker only |

```
Efficiency = max(0, 100 − sum(penalties))
```

Efficiency is **not** divided by top performer — it is personal.

---

## Clutch Bonus (raw)

Timed critical actions. Cap **+30 raw** per player per room.

| Action | Points | Condition |
|--------|--------|-----------|
| Last-second room clear contribution | +15 | Objective action in final 5s before clear |
| Save failing sub-task | +20 | Sub-task timer < 10s remaining |
| Sudden-death win | +30 | Wins tie-breaker for slot |
| 0-strike room | +10 | Efficiency ≥ 90 and Support ≥ 1 action |

---

## Composite score components

### Speed Score

```
Speed Score = min(100, 100 × (t_fastest / t_yours))
```

- `t_fastest` = fastest clear time among slots that cleared
- Slots that did not clear: `Speed Score = partial_progress_percent` (0–50 cap)

### Efficiency Score (slot-level)

**Solo:** player's Efficiency.

**Team:** average Efficiency of all living members.

### Cooperation Score (slot-level)

**Solo:** Support Points normalized to self (usually 0 unless future NPC assist rooms).

**Team:** sum of all members' Support Points, normalized against best team in lobby.

---

## Strikes

| Rule | Detail |
|------|--------|
| Who gets a Strike | Lowest CI on each **surviving** team in the danger zone (team modes only) |
| Strike limit | 2 Strikes → player eliminated before next room |
| Solo mode | Strikes disabled |
| AFK | Auto Strike + lowest CI flag |
| Grief flag | Auto Strike + matchmaking ban review |

---

## Finale payout split (wager mode)

### Solo — top 3 slots

Pool split by placement: **50% / 30% / 20%** of the 95% prize pool.

### Team — top 3 teams

Each team's share split among members by **finale CI**:

| Member rank (within team) | Share of team payout |
|---------------------------|----------------------|
| MVP (highest CI) | 40% |
| 2nd | 25% |
| 3rd | 20% |
| 4th | 15% |

Duo/trio use proportional split (50/50 duo; 40/35/25 trio by CI rank).

---

## Anti-grief & audit

| Check | Action |
|-------|--------|
| CI < 10 three rooms in a row | Matchmaking cooldown |
| Grief flag | 7-day wager ban, review |
| Collusion pattern (queue sniping) | Flagged accounts pooled separately |
| Score dispute | Server replay + point log (all raw events timestamped) |

---

## Server authority

All scoring runs **server-side**. Clients send intent events only:

- `Interact`, `Place`, `Flip`, `Carry`, `Revive`, `Ping`
- Server assigns points and replicates CI/composite to clients at 1 Hz during room, full snapshot on elimination

---

## Related

- [TOURNAMENT.md](TOURNAMENT.md) — when scores trigger elimination
- [ROOMS.md](ROOMS.md) — room-specific objective caps
- [WAGERING.md](WAGERING.md) — payout worked examples
