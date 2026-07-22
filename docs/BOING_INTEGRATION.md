# Boing Network integration

**PugdyMon: Party Saga** uses **[Boing Network](https://boing.network/)** for NFT skins and soft currency.

Boing is a **native L1** (32-byte `AccountId`, Ed25519, Boing VM) — not MetaMask / ERC-721.

## Config

| Source | Purpose |
|--------|---------|
| `BOING_RPC_URL` | JSON-RPC endpoint (default `http://127.0.0.1:8545`) |
| `BOING_ACCOUNT` | Your `0x` + 64 hex AccountId — Ctrl+V in-game to link |
| [`data/boing/contracts.json`](../data/boing/contracts.json) | Deployed NFT collection + fungible AccountIds |

Chain id (testnet): **6913** / `0x1b01`.

## Wallet

Use **[Boing Express](https://boing.express)** for signing. Desktop Bevy cannot inject `window.boing`; claim flow:

1. Earn season points in-game (M builds a voucher).
2. Voucher written to `%LOCALAPPDATA%/PugdyMon/logs/claim_voucher.json`.
3. Companion page / operator uses **boing-sdk** `buildReferenceNftCollectionDeployMetaTx` / mint `contract_call` with Express.

## Deploy reference assets

From a machine with `boing-sdk` (Boing Network repo):

```bash
# See scripts/boing/deploy_reference_assets.mjs
node scripts/boing/deploy_reference_assets.mjs
```

Paste resulting AccountIds into `data/boing/contracts.json`.

## Claim companion

Static page: [`companion/claim/`](../companion/claim/README.md)

In-game **Ctrl+O** opens it. Paste `%LOCALAPPDATA%/PugdyMon/logs/claim_voucher.json` and continue in Boing Express.

## In-game keys

| Key | Action |
|-----|--------|
| Ctrl+V | Link `BOING_ACCOUNT` |
| M | Write claim voucher for equipped skin |
| Ctrl+O | Open claim companion page |
| C | Cycle unlocked cosmetics |

## Security

- Season points are **host-trusted** offline/LAN for MVP.
- Do not mint from raw client scores without attestation later.
- NFTs/currency ≠ gambling; see archived wager docs.
