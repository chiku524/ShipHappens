#!/usr/bin/env python3
"""Validate studio registry ↔ GLB files ↔ room layout asset_ids.

Exit 0 when clean; exit 1 with a report of problems.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parents[1]
_REGISTRY = _REPO_ROOT / "assets" / "studio_registry.json"
_MODELS = _REPO_ROOT / "assets" / "models"
_ROOMS = _REPO_ROOT / "data" / "rooms"


def main() -> int:
    registry = json.loads(_REGISTRY.read_text(encoding="utf-8"))
    assets = {
        a["asset_id"]
        for a in registry.get("assets", [])
        if isinstance(a, dict) and a.get("asset_id")
    }

    missing_glb: list[str] = []
    for aid in sorted(assets):
        glb = _MODELS / aid / f"{aid}.glb"
        if not glb.is_file():
            missing_glb.append(aid)

    orphan_folders: list[str] = []
    if _MODELS.is_dir():
        for folder in sorted(p for p in _MODELS.iterdir() if p.is_dir()):
            if folder.name not in assets and (folder / f"{folder.name}.glb").is_file():
                orphan_folders.append(folder.name)

    unknown_refs: list[str] = []
    for path in sorted(_ROOMS.glob("*.json")):
        data = json.loads(path.read_text(encoding="utf-8"))
        for marker in data.get("markers", []):
            aid = marker.get("asset_id")
            if isinstance(aid, str) and aid and aid not in assets:
                unknown_refs.append(f"{path.name}::{marker.get('id')} → {aid}")

    problems = False
    if missing_glb:
        problems = True
        print("Registry entries missing GLB:")
        for aid in missing_glb:
            print(f"  - {aid}")
    if unknown_refs:
        problems = True
        print("Room markers reference unknown asset_ids:")
        for ref in unknown_refs:
            print(f"  - {ref}")
    if orphan_folders:
        print("GLB folders not in registry (informational):")
        for aid in orphan_folders:
            print(f"  - {aid}")

    if not problems:
        print(f"OK: {len(assets)} registry assets, room refs valid")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
