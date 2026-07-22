# Packaging & playtester builds

**PudgyMon: Party Saga** — Nest plaza, mini-games, season points, optional Boing Network rewards.

## Windows release folder

```powershell
pwsh scripts/build_windows_release.ps1
```

Output: `dist/PudgyMon/` with `pudgymon.exe`, `assets/`, `data/`, and `README.txt`.

### In-game

- Boots into **The Nest** (no main menu)
- Walk glowing pads: Race · Vibe · Shooter · Party Saga → **E** / Enter to start
- **Create Map** (orange) / **My Maps** (purple) — Race / Vibe / Shooter + Party Saga packs; see [MAP_CREATOR.md](MAP_CREATOR.md)
- **Esc** Nest menu (Settings · Profile · Account · Inventory · Wallet · Market · Challenges · Controls · Quit) — Esc again closes
- Accounts website: [`web/`](../web/) + API [`services/accounts/`](../services/accounts/) — see [ACCOUNTS.md](ACCOUNTS.md)
- **C** cycle skins · **M** claim voucher · **Ctrl+V** link `BOING_ACCOUNT`
- **Tab** spectate after race finish · **R** rematch · **Q** return to Nest
- Weekly challenges: `data/challenges/weekly.json`

### LAN multiplayer (Nest + stages)

```text
pudgymon.exe host --port 7777
pudgymon.exe join --address 192.168.1.10 --port 7777
```

Host is listen-server authority. Joiners see each other in **The Nest**, can press **E** on pads to queue a mode for everyone, and share Race / Vibe / Shooter (movement + scoring on host; shooters send fire to host). Custom My Maps need the same map JSON on host (and ideally joiners) under `%LOCALAPPDATA%/PudgyMon/maps/`.

## Crash / log path

`%LOCALAPPDATA%\PudgyMon\logs\crash.log`  
Claim vouchers: `%LOCALAPPDATA%\PudgyMon\logs\claim_voucher.json`

## Boing

See [BOING_INTEGRATION.md](BOING_INTEGRATION.md).

## Steam

See [STEAM.md](STEAM.md).
