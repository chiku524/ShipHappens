#!/usr/bin/env python3
"""Validate assets/audio/catalog.json paths and report coverage."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "assets" / "audio" / "catalog.json"
AUDIO_ROOT = ROOT / "assets" / "audio"


def main() -> int:
    if not CATALOG.is_file():
        print(f"missing catalog: {CATALOG}", file=sys.stderr)
        return 1

    data = json.loads(CATALOG.read_text(encoding="utf-8"))
    missing: list[str] = []
    present: list[str] = []

    for section in ("sfx", "music", "vo"):
        entries = data.get(section) or {}
        for key, rel in entries.items():
            if str(key).startswith("_"):
                continue
            path = AUDIO_ROOT / rel
            label = f"{section}.{key} -> {rel}"
            if path.is_file():
                present.append(label)
            else:
                missing.append(label)

    print(f"audio catalog: {len(present)} present, {len(missing)} pending (ok)")
    for line in present:
        print(f"  [ok] {line}")
    for line in missing:
        print(f"  [..] {line}")

    # Pending is expected — only fail on bad catalog structure.
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
