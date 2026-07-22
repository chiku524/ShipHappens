# Start the free Cloudflare named tunnel that exposes local accounts API.
# Prerequisites:
#   1) cd services/accounts; copy .env.example .env; docker compose up -d
#   2) cloudflared installed and logged in

$ErrorActionPreference = "Stop"
$config = Join-Path $env:USERPROFILE ".cloudflared\config.yml"
if (-not (Test-Path $config)) {
    Write-Error "Missing $config — see services/accounts/cloudflared.yml"
}

Write-Host "Exposing http://127.0.0.1:8788 as https://pudgymon-api.boing.network"
& cloudflared tunnel --config $config run boing-testnet-rpc
