# Wager Economy & Prize Pools

> **SUPERSEDED:** PudgyMon: Party Saga uses **season points + Boing Network NFTs/currency** (collectibles), not real-money prize pools. See [PARTY_ROADMAP.md](PARTY_ROADMAP.md) and [BOING_INTEGRATION.md](BOING_INTEGRATION.md). No stake-to-win until legal review. The text below is archived vault-era design only.

## Overview

Wager tournaments allow players to buy into a bracket. The platform takes a **5% rake**; **95%** funds the prize pool distributed to **top 3** finishers.

> **Compliance:** Real-money wagering is subject to gambling laws, age verification (18+/21+), geo-blocking, KYC, and responsible-gaming requirements. **MVP ships with practice currency only.** Wager mode is a gated phase after skill integrity and legal review.

## Account wallet

| Rule | Value |
|------|-------|
| Initial deposit cap | $10 per account (tunable) |
| Weekly deposit limit | $10 per rolling 7 days |
| Weekly loss limit | $25 (hard stop; cannot queue wager) |
| Minimum withdrawal | $5 |
| Withdrawal processing | T+3 business days (placeholder) |

## Buy-in tiers

| Tier | Buy-in | Typical bracket |
|------|--------|-----------------|
| Intern | $1 | Solo 16 |
| Contractor | $5 | Solo 16 / Squad 8 |
| Executive | $10 | Solo 16 / Squad 8 |

Buy-in is locked at lobby start. No top-ups mid-tournament.

## Prize pool math

```
Gross pool   = buy_in × number_of_slots
Platform rake = Gross pool × 0.05
Prize pool   = Gross pool × 0.95
```

### Solo 16, $5 Contractor example

| Line | Amount |
|------|--------|
| Gross (16 × $5) | $80.00 |
| Rake (5%) | $4.00 |
| Prize pool (95%) | $76.00 |
| 1st (50%) | $38.00 |
| 2nd (30%) | $22.80 |
| 3rd (20%) | $15.20 |

### Squad 8 (4 players each), $5 per player

| Line | Amount |
|------|--------|
| Gross (8 teams × 4 × $5) | $160.00 |
| Rake (5%) | $8.00 |
| Prize pool (95%) | $152.00 |
| 1st team | $76.00 |
| 2nd team | $45.60 |
| 3rd team | $30.40 |

Internal team split by finale CI — see [SCORING.md](SCORING.md#finale-payout-split-wager-mode).

## Practice currency (MVP)

| Feature | Practice | Wager |
|---------|----------|-------|
| Currency | Vault Credits (VC) | USD wallet |
| Rake | 0% | 5% |
| Payout | Cosmetic only | Real money |
| Queue unlock | Immediate | After rank + age gate |

Practice uses **identical** bracket and scoring math so players learn payout expectations.

## Wager queue gates

Before accessing real-money queues:

| Requirement | Threshold |
|-------------|-----------|
| Account age | 7 days |
| Practice games played | ≥ 20 |
| Practice rank | Silver or higher |
| Age verification | Passed (ID check) |
| Geo | Allowed jurisdiction only |
| Loss limit acknowledged | User confirms weekly cap |

## Anti-abuse

| Threat | Mitigation |
|--------|------------|
| Collusion (queue sniping) | Separate flagged pool; IP/device fingerprint review |
| Smurfing | Wallet + device linkage; minimum account age |
| AFK farming | Auto Strike; no payout below CI floor |
| Griefing | Wager ban 7–30 days |
| Money laundering | Withdrawal KYC; deposit source checks |
| Underage | ID verification before first deposit |

## Handshake side bets (future)

Two slots may wager **10% of their own max payout** on head-to-head placement in the next room. Platform takes 5% of the side pot. Opt-in only; both must accept before room start.

## Spectator revenue (future)

Cosmetic "cheer" effects add to a **spectator cosmetic pool** — does not increase competitive prize pool. Avoid pay-to-influence outcomes.

## Treasury Ghost (5% rake fiction)

In-universe, the 5% funds the **Treasury Ghost** — HR AI that hosts ongoing seasonal cups and maintenance. Marketing copy; actual ops follow legal accounting.

## Responsible gaming

- Weekly loss limit (default $25)
- Session time reminders every 60 minutes in wager mode
- Self-exclusion: 30 / 90 / 365 days
- Links to problem-gambling resources in wallet UI

## Related

- [TOURNAMENT.md](TOURNAMENT.md) — bracket structure
- [SCORING.md](SCORING.md) — payout splits
- [GDD.md](GDD.md) — design summary
