# Audio drop-in

Place files next to this README using the paths in `catalog.json`.

| Folder | Purpose | When it plays |
|--------|---------|---------------|
| `sfx/` | One-shots | Interact juice (pickup, sort, zap, clear, fail) |
| `music/` | Looping beds | Title, lobby, each vault, elimination, podium |
| `vo/` | PA / announcer | Room start, clear, wrong sort, meltdown, elimination, podium |

**Formats:** `.ogg` preferred (also `.wav` / `.mp3` if you rename paths in `catalog.json`).

**Rules:**
1. Keep catalog keys stable — only change the path string if you rename a file.
2. Missing files are fine: SFX falls back to sine tones; music/VO stay silent.
3. After dropping files, restart the game (catalog is loaded at startup).

Validate:

```bash
python scripts/validate_audio_catalog.py
```
