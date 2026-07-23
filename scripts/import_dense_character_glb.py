#!/usr/bin/env python3
"""Import a dense Tripo/download GLB as a playable Pudgy character.

Decimates high-poly scans, floor-pivots, bakes ~1.2 m height, adds accessory sockets.

Usage:
  python scripts/import_dense_character_glb.py \\
    --src "C:/Users/.../stylized+pink+creature+3d+model.glb" \\
    --asset-id char_pudgy_pink_01
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_MODELS = _REPO / "assets" / "models"
_REGISTRY = _REPO / "assets" / "studio_registry.json"
_BLENDER = Path(r"C:\Program Files\Blender Foundation\Blender 5.1\blender.exe")

_WORKER = r'''
import bpy
import math
import mathutils
from pathlib import Path

IN_PATH = Path(r"__IN_PATH__")
OUT_PATH = Path(r"__OUT_PATH__")
ASSET_ID = "__ASSET_ID__"
TARGET_HEIGHT = float("__TARGET_HEIGHT__")
TARGET_FACES = int("__TARGET_FACES__")

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.ops.import_scene.gltf(filepath=str(IN_PATH))

meshes = [o for o in bpy.context.scene.objects if o.type == "MESH"]
if not meshes:
    raise RuntimeError("no mesh in GLB")

bpy.ops.object.select_all(action="DESELECT")
for o in meshes:
    o.select_set(True)
bpy.context.view_layer.objects.active = meshes[0]
if len(meshes) > 1:
    bpy.ops.object.join()
body = bpy.context.view_layer.objects.active
body.name = ASSET_ID
if body.data:
    body.data.name = ASSET_ID

bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)

# Decimate dense Tripo exports for runtime.
face_count = len(body.data.polygons)
if face_count > TARGET_FACES:
    ratio = max(TARGET_FACES / float(face_count), 0.001)
    mod = body.modifiers.new(name="GameDecimate", type="DECIMATE")
    mod.ratio = ratio
    bpy.ops.object.modifier_apply(modifier=mod.name)
    print("DECIMATE", face_count, "->", len(body.data.polygons), "ratio", round(ratio, 5))

# Soft matte cartoon defaults on unlinked inputs
for slot in body.material_slots:
    mat = slot.material
    if not mat or not mat.use_nodes:
        continue
    nt = mat.node_tree
    principled = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
    if not principled:
        continue
    if "Roughness" in principled.inputs and not principled.inputs["Roughness"].is_linked:
        principled.inputs["Roughness"].default_value = 0.62
    if "Coat Weight" in principled.inputs and not principled.inputs["Coat Weight"].is_linked:
        principled.inputs["Coat Weight"].default_value = 0.0
    if "Metallic" in principled.inputs and not principled.inputs["Metallic"].is_linked:
        principled.inputs["Metallic"].default_value = 0.0
    mat.blend_method = "OPAQUE"

def world_aabb(obj):
    minv = mathutils.Vector((1e9, 1e9, 1e9))
    maxv = mathutils.Vector((-1e9, -1e9, -1e9))
    for corner in obj.bound_box:
        w = obj.matrix_world @ mathutils.Vector(corner)
        minv = mathutils.Vector((min(minv.x, w.x), min(minv.y, w.y), min(minv.z, w.z)))
        maxv = mathutils.Vector((max(maxv.x, w.x), max(maxv.y, w.y), max(maxv.z, w.z)))
    return minv, maxv

minv, maxv = world_aabb(body)
h = max(maxv.z - minv.z, 1e-4)
scale = TARGET_HEIGHT / h
body.scale = (scale, scale, scale)
bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)

minv, maxv = world_aabb(body)
cx = 0.5 * (minv.x + maxv.x)
cy = 0.5 * (minv.y + maxv.y)
body.location -= mathutils.Vector((cx, cy, minv.z))
bpy.ops.object.transform_apply(location=True, rotation=False, scale=False)

minv, maxv = world_aabb(body)
h = maxv.z - minv.z
w = maxv.x - minv.x
d = maxv.y - minv.y
cx = (minv.x + maxv.x) * 0.5
cy = (minv.y + maxv.y) * 0.5

sockets = {
    "Socket_Hat": (cx, cy, maxv.z * 0.98),
    "Socket_Necklace": (cx, cy - d * 0.05, h * 0.62),
    "Socket_Shoes": (cx, cy, 0.02),
    "Socket_Back": (cx, cy + d * 0.35, h * 0.55),
    "Socket_Face": (cx, cy - d * 0.42, h * 0.78),
    "Socket_Hands": (cx, cy, h * 0.42),
}

root = bpy.data.objects.new(f"{ASSET_ID}_Root", None)
bpy.context.scene.collection.objects.link(root)

def parent_keep(child, parent):
    mw = child.matrix_world.copy()
    child.parent = parent
    child.matrix_world = mw

parent_keep(body, root)
for name, loc in sockets.items():
    empty = bpy.data.objects.new(name, None)
    empty.empty_display_type = "PLAIN_AXES"
    empty.empty_display_size = 0.08
    empty.location = loc
    bpy.context.scene.collection.objects.link(empty)
    parent_keep(empty, root)

OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
bpy.ops.export_scene.gltf(
    filepath=str(OUT_PATH),
    export_format="GLB",
    use_selection=False,
    export_apply=True,
    export_texcoords=True,
    export_normals=True,
    export_materials="EXPORT",
    export_image_format="AUTO",
    export_yup=True,
)
minv, maxv = world_aabb(body)
print("IMPORT_OK", ASSET_ID)
print("height", round(maxv.z - minv.z, 4))
print("faces", len(body.data.polygons))
'''


def _register(asset_id: str, notes: str) -> None:
    registry = {"import_root": "res://assets/models", "assets": []}
    if _REGISTRY.is_file():
        registry = json.loads(_REGISTRY.read_text(encoding="utf-8"))
    by_id = {
        a["asset_id"]: a
        for a in registry.get("assets", [])
        if isinstance(a, dict) and a.get("asset_id")
    }
    by_id[asset_id] = {
        "asset_id": asset_id,
        "target_height": 1.2,
        "uniform_scale": 1.0,
        "notes": notes,
    }
    registry["assets"] = sorted(by_id.values(), key=lambda x: x["asset_id"])
    _REGISTRY.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--src", type=Path, required=True)
    parser.add_argument("--asset-id", required=True)
    parser.add_argument("--height", type=float, default=1.2)
    parser.add_argument("--faces", type=int, default=28000)
    parser.add_argument("--notes", default="Imported dense creature GLB (decimated for play).")
    args = parser.parse_args()

    if not _BLENDER.is_file():
        print("error: Blender not found", file=sys.stderr)
        return 1
    if not args.src.is_file():
        print(f"error: missing {args.src}", file=sys.stderr)
        return 1

    aid = args.asset_id.strip()
    dest_dir = _MODELS / aid
    dest_dir.mkdir(parents=True, exist_ok=True)
    out = dest_dir / f"{aid}.glb"
    worker = dest_dir / "_import_dense_worker.py"
    script = (
        _WORKER.replace("__IN_PATH__", str(args.src.resolve()).replace("\\", "/"))
        .replace("__OUT_PATH__", str(out.resolve()).replace("\\", "/"))
        .replace("__ASSET_ID__", aid)
        .replace("__TARGET_HEIGHT__", str(args.height))
        .replace("__TARGET_FACES__", str(args.faces))
    )
    worker.write_text(script, encoding="utf-8")
    try:
        proc = subprocess.run(
            [str(_BLENDER), "--background", "--python", str(worker)],
            capture_output=True,
            text=True,
        )
    finally:
        worker.unlink(missing_ok=True)

    print(proc.stdout[-3000:] if proc.stdout else "")
    if proc.returncode != 0 or not out.is_file():
        print(proc.stderr[-4000:], file=sys.stderr)
        return 1

    (dest_dir / "README.txt").write_text(
        f"{aid}\nSource: {args.src.name}\nDecimated playable import (~{args.faces} faces, height {args.height}).\n",
        encoding="utf-8",
    )
    _register(aid, args.notes)
    print(f"glb -> {out.relative_to(_REPO)} ({out.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
