# Build a playtester-ready Windows folder (no Rust required to run).
# Usage: pwsh scripts/build_windows_release.ps1

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $Root

Write-Host "Building release…"
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$Out = Join-Path $Root "dist\PugdyMon"
if (Test-Path $Out) { Remove-Item -Recurse -Force $Out }
New-Item -ItemType Directory -Path $Out | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Out "assets") | Out-Null
New-Item -ItemType Directory -Path (Join-Path $Out "data") | Out-Null

Copy-Item (Join-Path $Root "target\release\pudgymon.exe") $Out
Copy-Item -Recurse (Join-Path $Root "assets\*") (Join-Path $Out "assets")
Copy-Item -Recurse (Join-Path $Root "data\*") (Join-Path $Out "data")

@"
PugdyMon: Party Saga (practice build)

Run: pudgymon.exe

Boots into The Nest — walk a glowing pad, press E to play.
Pads: Race · Vibe · Shooter · Party Saga
Controls: WASD · C skins · M Boing claim · Esc pause · R rematch · Q Nest

Crash logs: %LOCALAPPDATA%\PugdyMon\logs\crash.log

Host for friends: pudgymon.exe host --port 7777
Join:           pudgymon.exe join --address <HOST_IP> --port 7777
"@ | Set-Content -Path (Join-Path $Out "README.txt") -Encoding UTF8

Write-Host "Ready: $Out"
Write-Host "Zip that folder for playtesters."
