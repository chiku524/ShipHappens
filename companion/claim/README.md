# PugdyMon Claim Companion

Static page for the desktop → Boing Express claim path.

## Run locally

```bash
# from repo root
npx --yes serve companion/claim -p 4173
```

Open `http://127.0.0.1:4173`.

## Flow

1. In game: link wallet (Ctrl+V + `BOING_ACCOUNT`), equip skin, press **M**.
2. Open `%LOCALAPPDATA%/PugdyMon/logs/claim_voucher.json`.
3. Paste or load into this page → Parse.
4. Open Boing Express (chain **6913**) and complete mint against `data/boing/contracts.json`.

Until the NFT collection is deployed, **Try Express mint** only sends a claim intent placeholder when `window.boing` is present.
