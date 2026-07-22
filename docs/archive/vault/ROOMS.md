# Escape Rooms (Vault Stages)

## Room archetypes

ShipHappens rotates four archetypes across a tournament. Each archetype scales by slot size.

| Archetype | Co-op pattern | Example |
|-----------|---------------|---------|
| **Parallel pressure** | Sub-tasks must complete within time window | Valve squad + key runner |
| **Information asymmetry** | Each player sees different clues | Breaker labels, wire diagrams |
| **Physics comedy** | Slapstick coordination | Gantry aim / lift / catch |
| **Finale meltdown** | Shared meter + individual tracking | Coolant + load + seal |

Future archetypes (post-MVP): social deduction lite (glitch role), double-or-nothing side room, remnant clue overlays.

---

## Room scaling

Same blueprint, scaled objectives:

| Slot size | Scaling rule |
|-----------|--------------|
| Solo | 1× objectives; player performs all roles |
| Duo | 2× parallel tracks OR 50% objectives each |
| Trio | 3 sub-tasks OR split 4th role as rotating "runner" |
| Squad | 4 parallel sub-tasks + shared finale beat |

### Example: Breaker Panic (12 switches)

| Slot | Layout |
|------|--------|
| Solo | 12 switches, one panel |
| Duo | 6 + 6 on two panels; sequence shared via pings |
| Trio | 4 + 4 + 4 |
| Squad | 3 + 3 + 3 + 3 |

---

## MVP tournament rooms

### Room 1 — HR Orientation Bay

**Archetype:** Parallel pressure (light tutorial)  
**Duration:** 5 min  
**Tone:** *"Please sort your soul— I mean, freight— into the correct chute."*

**Objective:** Sort 24 absurd freight items into 4 labeled chutes (Hot Dogs, Sentient Toasters, Premium Air, Misc Write-Ups).

| Slot | Scaling |
|------|---------|
| Solo | Sort all 24 |
| Duo | 12 each, shared conveyor |
| Trio | 8 each |
| Squad | 6 each + one "scanner" role (optional Leaseholder) |

**Scoring highlights:**
- +8 correct, −3 wrong per item
- Conveyor speeds up every 8 correct sorts

**Elimination:** Bottom 25% composite (16 → 12 solo).

**Assets (existing):** freight crates, kiosk props, cargo signage.

---

### Room 2 — Cargo Ring Gantry

**Archetype:** Physics comedy  
**Duration:** 6 min  
**Tone:** *"The crane is older than your contract. Do not drop the eggs. (They are explosive.)"*

**Objective:** Deliver 12 marked crates from intake to cargo ring bay.

| Role | Action |
|------|--------|
| **Aimer** | Rotate crane hook |
| **Lifter** | Raise / lower |
| **Catcher** | Guide crate into zone (solo performs all) |

| Slot | Scaling |
|------|---------|
| Solo | 12 crates, all roles |
| Duo | 6 crates each OR alternating roles |
| Trio | 4 crates each + rotate roles every 2 crates |
| Squad | 3 crates each; fixed roles |

**Scoring highlights:**
- +15 delivery, −8 drop, +12 catch teammate drop
- Write-Up Meter: 3 drops = team efficiency penalty tier

**Elimination:** Bottom 33% of remaining.

**Assets (existing):** `env_cargo_crane_operator_console_01`, `env_cargo_gantry_hook_01`, freight crates.

---

### Room 3 — Breaker Panic

**Archetype:** Information asymmetry  
**Duration:** 6 min  
**Tone:** *"Power Hour is mandatory. The sequence is in your heart. And on someone else's panel."*

**Objective:** Flip 12 breakers in the correct sequence. Each player sees **only their panel labels**; sequence is derived from combined clues.

| Slot | Scaling |
|------|---------|
| Solo | Full sequence, 12 flips |
| Duo | 6 + 6; must communicate order |
| Trio | 4 + 4 + 4 |
| Squad | 3 each; fourth player is Leaseholder (directs, no flip) |

**Scoring highlights:**
- +12 correct flip, −8 wrong (cartoon zap ragdoll)
- Wrong flip adds +2s team stun

**Elimination:** Bottom 50% of remaining. **Strikes** assigned to lowest CI on surviving danger-zone teams.

**Assets (existing):** `env_breaker_panel_01`, Power Hour sequence pattern from legacy design `[0, 2, 1, 3]` extended for 12 switches.

---

### Finale — Shuttle Bay Meltdown

**Archetype:** Finale meltdown  
**Duration:** 7 min  
**Tone:** *"The shuttle leaves in seven minutes. So does anyone who isn't loading coolant."*

**Objective:** Shared **Meltdown Meter** (0–100). Team/solo must keep meter below 100 while completing:

1. Turn 6 coolant valves (+10 each)
2. Seal 3 doors (+15 each)
3. Load 4 escape crates (+20 each)

Meter rises +2/s baseline; wrong breaker-style mistakes +15 spike.

**Ranking:** All finalists clear or fail together; **individual CI** ranks payout (solo: 1st–3rd; teams: team placement then internal split).

**Assets (existing):** coolant props, shuttle bay escape zone, freight crates.

---

## Future room pool (season 1+)

| Room | Archetype | Hook |
|------|-----------|------|
| **Mirror Maze** | Information asymmetry | Scout sees path; others hear distorted directions |
| **Pipe Dream** | Parallel pressure | Valve order; wrong order floods room |
| **Vault Lock** | Parallel pressure | 4 simultaneous key mini-games |
| **Glitch Socket** | Social deduction lite | One harmless saboteur per room |
| **Double-or-Nothing Airlock** | Optional side room | Skip seed vs elimination risk |

---

## Leaseholder mechanic (team rooms)

Every team room can enable **The Lease**:

- One player rotated each room
- Leaseholder sees **full objective text** and minimap markers
- Leaseholder **cannot interact** with objects — only ping and voice/text callouts
- +8 Support per validated callout that leads to objective within 5s

Rotation order: lowest Support CI from prior room goes Leaseholder (gives underperformers a recovery role).

---

## Remnant clues

Eliminated slots project a **holographic hint** in the next room for survivors:

- One incorrect breaker label grayed out, OR
- One wrong sort chute marked with skull icon

Effect: leaders lose ~3s average; underdogs gain information. Logged so wager audits can verify no collusion.

---

## Related

- [TOURNAMENT.md](TOURNAMENT.md) — elimination cadence
- [SCORING.md](SCORING.md) — point values
- [GDD.md](GDD.md) — design pillars
