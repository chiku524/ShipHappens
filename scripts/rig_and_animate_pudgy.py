#!/usr/bin/env python3
"""Rig a playable Pudgy GLB with the shared armature + party animation clips.

Pipeline:
  1. UV-aware simplify (~100k faces) via gltf-transform
  2. Blender: shared bones, automatic weights, sockets on bones
  3. Author idle/walk/run/jump/emote_wave/emote_dance
  4. Export Bevy-safe skinned GLB (no Draco/WebP)

Usage:
  python scripts/rig_and_animate_pudgy.py --asset-id char_pudgy_pink_01
  python scripts/rig_and_animate_pudgy.py --asset-id char_pudgy_stylized_01
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_MODELS = _REPO / "assets" / "models"
_REGISTRY = _REPO / "assets" / "studio_registry.json"
_BLENDER = Path(r"C:\Program Files\Blender Foundation\Blender 5.1\blender.exe")

_CLIP_NAMES = (
    "idle",
    "walk",
    "run",
    "jump",
    "emote_wave",
    "emote_dance",
)

_WORKER = r'''
import bpy
import math
import mathutils
from pathlib import Path

IN_PATH = Path(r"__IN_PATH__")
OUT_PATH = Path(r"__OUT_PATH__")
ASSET_ID = "__ASSET_ID__"
FPS = 24

bpy.ops.wm.read_factory_settings(use_empty=True)
bpy.context.scene.render.fps = FPS
bpy.ops.import_scene.gltf(filepath=str(IN_PATH))

# Collect mesh + existing sockets
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

# Clean topology so automatic weights succeed on Tripo meshes.
bpy.context.view_layer.objects.active = body
bpy.ops.object.mode_set(mode="EDIT")
bpy.ops.mesh.select_all(action="SELECT")
bpy.ops.mesh.remove_doubles(threshold=0.0001)
bpy.ops.mesh.normals_make_consistent(inside=False)
bpy.ops.object.mode_set(mode="OBJECT")

sockets = {
    o.name: o
    for o in bpy.context.scene.objects
    if o.type == "EMPTY" and o.name.startswith("Socket_")
}

# Opaque materials
for slot in body.material_slots:
    mat = slot.material
    if not mat:
        continue
    if hasattr(mat, "blend_method"):
        mat.blend_method = "OPAQUE"
    if not getattr(mat, "use_nodes", False):
        continue
    nt = mat.node_tree
    principled = next((n for n in nt.nodes if n.type == "BSDF_PRINCIPLED"), None)
    if not principled:
        continue
    if "Alpha" in principled.inputs:
        for link in list(principled.inputs["Alpha"].links):
            nt.links.remove(link)
        principled.inputs["Alpha"].default_value = 1.0

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
w = max(maxv.x - minv.x, 1e-4)
d = max(maxv.y - minv.y, 1e-4)
cx = 0.5 * (minv.x + maxv.x)
cy = 0.5 * (minv.y + maxv.y)

# --- Shared Pudgy armature (contract + quality extras) ---
arm_data = bpy.data.armatures.new(f"{ASSET_ID}_Armature")
arm_obj = bpy.data.objects.new(f"{ASSET_ID}_Armature", arm_data)
bpy.context.scene.collection.objects.link(arm_obj)
bpy.context.view_layer.objects.active = arm_obj
arm_obj.select_set(True)
bpy.ops.object.mode_set(mode="EDIT")
eb = arm_data.edit_bones

def add_bone(name, head, tail, parent=None):
    b = eb.new(name)
    b.head = head
    b.tail = tail
    if parent is not None:
        b.parent = parent
    return b

# Blender import is often Z-up from glTF; our prior polish bakes floor on Z.
# Use Z as up for bone placement to match the imported mesh AABB.
root = add_bone("Root", (cx, cy, 0.0), (cx, cy, h * 0.08))
hips = add_bone("Hips", (cx, cy, h * 0.28), (cx, cy, h * 0.40), root)
spine = add_bone("Spine", (cx, cy, h * 0.40), (cx, cy, h * 0.62), hips)
head = add_bone("Head", (cx, cy, h * 0.62), (cx, cy, h * 0.92), spine)

arm_y = cy
l_arm = add_bone("L_Arm", (cx + w * 0.18, arm_y, h * 0.58), (cx + w * 0.32, arm_y - d * 0.05, h * 0.48), spine)
l_fore = add_bone("L_Forearm", (cx + w * 0.32, arm_y - d * 0.05, h * 0.48), (cx + w * 0.38, arm_y - d * 0.08, h * 0.36), l_arm)
r_arm = add_bone("R_Arm", (cx - w * 0.18, arm_y, h * 0.58), (cx - w * 0.32, arm_y - d * 0.05, h * 0.48), spine)
r_fore = add_bone("R_Forearm", (cx - w * 0.32, arm_y - d * 0.05, h * 0.48), (cx - w * 0.38, arm_y - d * 0.08, h * 0.36), r_arm)

l_leg = add_bone("L_Leg", (cx + w * 0.10, cy, h * 0.28), (cx + w * 0.12, cy, h * 0.14), hips)
l_shin = add_bone("L_Shin", (cx + w * 0.12, cy, h * 0.14), (cx + w * 0.12, cy, 0.02), l_leg)
r_leg = add_bone("R_Leg", (cx - w * 0.10, cy, h * 0.28), (cx - w * 0.12, cy, h * 0.14), hips)
r_shin = add_bone("R_Shin", (cx - w * 0.12, cy, h * 0.14), (cx - w * 0.12, cy, 0.02), r_leg)

bpy.ops.object.mode_set(mode="OBJECT")

# Parent mesh with automatic weights
bpy.ops.object.select_all(action="DESELECT")
body.select_set(True)
arm_obj.select_set(True)
bpy.context.view_layer.objects.active = arm_obj
bpy.ops.object.parent_set(type="ARMATURE_AUTO")

def count_unweighted():
    bad = []
    for vert in body.data.vertices:
        total = sum(g.weight for g in vert.groups)
        if total < 1e-4:
            bad.append(vert.index)
    return bad

unweighted = count_unweighted()
vert_count = max(len(body.data.vertices), 1)
if len(body.vertex_groups) < 2 or (len(unweighted) / vert_count) > 0.05:
    print(
        "AUTO weights poor (",
        len(unweighted),
        "/",
        vert_count,
        "); retrying ENVELOPE",
    )
    bpy.ops.object.parent_clear(type="CLEAR_KEEP_TRANSFORM")
    # Clear old groups
    body.vertex_groups.clear()
    bpy.ops.object.select_all(action="DESELECT")
    body.select_set(True)
    arm_obj.select_set(True)
    bpy.context.view_layer.objects.active = arm_obj
    # Enlarge envelopes slightly for chunky mascots
    bpy.ops.object.mode_set(mode="POSE")
    for pb in arm_obj.pose.bones:
        pb.bone.envelope_distance = max(pb.bone.envelope_distance, 0.35)
        pb.bone.head_radius = max(pb.bone.head_radius, 0.12)
        pb.bone.tail_radius = max(pb.bone.tail_radius, 0.10)
    bpy.ops.object.mode_set(mode="OBJECT")
    bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")
    unweighted = count_unweighted()

print("weight groups", len(body.vertex_groups), "unweighted", len(unweighted))

# Final repair: any remaining zero-weight verts → Hips (avoids skinned holes).
hips_vg = body.vertex_groups.get("Hips")
if hips_vg is None:
    hips_vg = body.vertex_groups.new(name="Hips")
if unweighted:
    hips_vg.add(unweighted, 1.0, "REPLACE")
    print("WEIGHT_REPAIR", len(unweighted), "verts -> Hips")

# Parent sockets to bones (keep world transform)
SOCKET_BONE = {
    "Socket_Hat": "Head",
    "Socket_Face": "Head",
    "Socket_Necklace": "Spine",
    "Socket_Back": "Spine",
    "Socket_Hands": "Spine",
    "Socket_Shoes": "Root",
}

def parent_to_bone(obj, armature, bone_name):
    mw = obj.matrix_world.copy()
    obj.parent = armature
    obj.parent_type = "BONE"
    obj.parent_bone = bone_name
    obj.matrix_world = mw

for sname, bone in SOCKET_BONE.items():
    if sname in sockets:
        parent_to_bone(sockets[sname], arm_obj, bone)

# Ensure a scene root empty for stable glTF hierarchy (optional keep arm as root)
# --- Animation helpers ---
def ensure_action(name):
    action = bpy.data.actions.get(name) or bpy.data.actions.new(name)
    action.use_fake_user = True
    return action

def set_bone_keys(action, bone_name, frames_euler):
    """frames_euler: list of (frame, (rx, ry, rz degrees)) plus optional location delta."""
    arm_obj.animation_data_create()
    arm_obj.animation_data.action = action
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="POSE")
    pb = arm_obj.pose.bones.get(bone_name)
    if pb is None:
        bpy.ops.object.mode_set(mode="OBJECT")
        return
    pb.rotation_mode = "XYZ"
    for item in frames_euler:
        if len(item) == 2:
            frame, eul = item
            loc = None
        else:
            frame, eul, loc = item
        bpy.context.scene.frame_set(int(frame))
        pb.rotation_euler = (
            math.radians(eul[0]),
            math.radians(eul[1]),
            math.radians(eul[2]),
        )
        pb.keyframe_insert(data_path="rotation_euler", frame=frame)
        if loc is not None:
            pb.location = mathutils.Vector(loc)
            pb.keyframe_insert(data_path="location", frame=frame)
    bpy.ops.object.mode_set(mode="OBJECT")

def clear_pose():
    bpy.context.view_layer.objects.active = arm_obj
    bpy.ops.object.mode_set(mode="POSE")
    bpy.ops.pose.select_all(action="SELECT")
    bpy.ops.pose.transforms_clear()
    bpy.ops.object.mode_set(mode="OBJECT")

# idle — gentle breathe (~2s = 48 frames)
clear_pose()
idle = ensure_action("idle")
set_bone_keys(idle, "Hips", [
    (1, (0, 0, 0), (0, 0, 0)),
    (24, (2, 0, 0), (0, 0, 0.012)),
    (48, (0, 0, 0), (0, 0, 0)),
])
set_bone_keys(idle, "Spine", [
    (1, (0, 0, 0)),
    (24, (-3, 0, 0)),
    (48, (0, 0, 0)),
])
set_bone_keys(idle, "Head", [
    (1, (0, 0, 0)),
    (24, (4, 0, 3)),
    (48, (0, 0, 0)),
])
set_bone_keys(idle, "L_Arm", [(1, (8, 0, 12)), (24, (10, 0, 14)), (48, (8, 0, 12))])
set_bone_keys(idle, "R_Arm", [(1, (8, 0, -12)), (24, (10, 0, -14)), (48, (8, 0, -12))])

# walk — ~0.8s = 20 frames
clear_pose()
walk = ensure_action("walk")
set_bone_keys(walk, "Hips", [
    (1, (0, 0, 0), (0, 0, 0.0)),
    (5, (0, 6, 0), (0, 0, 0.02)),
    (10, (0, 0, 0), (0, 0, 0.0)),
    (15, (0, -6, 0), (0, 0, 0.02)),
    (20, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(walk, "Spine", [
    (1, (4, 0, 0)), (10, (6, 0, 0)), (20, (4, 0, 0)),
])
set_bone_keys(walk, "L_Leg", [
    (1, (-28, 0, 0)), (10, (28, 0, 0)), (20, (-28, 0, 0)),
])
set_bone_keys(walk, "L_Shin", [
    (1, (18, 0, 0)), (5, (35, 0, 0)), (10, (8, 0, 0)), (20, (18, 0, 0)),
])
set_bone_keys(walk, "R_Leg", [
    (1, (28, 0, 0)), (10, (-28, 0, 0)), (20, (28, 0, 0)),
])
set_bone_keys(walk, "R_Shin", [
    (1, (8, 0, 0)), (10, (18, 0, 0)), (15, (35, 0, 0)), (20, (8, 0, 0)),
])
set_bone_keys(walk, "L_Arm", [
    (1, (20, 0, 25)), (10, (-15, 0, 20)), (20, (20, 0, 25)),
])
set_bone_keys(walk, "R_Arm", [
    (1, (-15, 0, -20)), (10, (20, 0, -25)), (20, (-15, 0, -20)),
])
set_bone_keys(walk, "Head", [
    (1, (0, 0, -4)), (10, (0, 0, 4)), (20, (0, 0, -4)),
])

# run — ~0.5s = 12 frames
clear_pose()
run = ensure_action("run")
set_bone_keys(run, "Hips", [
    (1, (8, 0, 0), (0, 0, 0.03)),
    (4, (8, 10, 0), (0, 0, 0.05)),
    (7, (8, 0, 0), (0, 0, 0.03)),
    (10, (8, -10, 0), (0, 0, 0.05)),
    (12, (8, 0, 0), (0, 0, 0.03)),
])
set_bone_keys(run, "Spine", [(1, (12, 0, 0)), (7, (14, 0, 0)), (12, (12, 0, 0))])
set_bone_keys(run, "L_Leg", [(1, (-40, 0, 0)), (7, (40, 0, 0)), (12, (-40, 0, 0))])
set_bone_keys(run, "L_Shin", [(1, (25, 0, 0)), (4, (50, 0, 0)), (7, (10, 0, 0)), (12, (25, 0, 0))])
set_bone_keys(run, "R_Leg", [(1, (40, 0, 0)), (7, (-40, 0, 0)), (12, (40, 0, 0))])
set_bone_keys(run, "R_Shin", [(1, (10, 0, 0)), (7, (25, 0, 0)), (10, (50, 0, 0)), (12, (10, 0, 0))])
set_bone_keys(run, "L_Arm", [(1, (35, 0, 35)), (7, (-30, 0, 25)), (12, (35, 0, 35))])
set_bone_keys(run, "R_Arm", [(1, (-30, 0, -25)), (7, (35, 0, -35)), (12, (-30, 0, -25))])

# jump — one-shot ~0.6s = 15 frames
clear_pose()
jump = ensure_action("jump")
set_bone_keys(jump, "Hips", [
    (1, (15, 0, 0), (0, 0, -0.04)),
    (4, (-10, 0, 0), (0, 0, 0.08)),
    (8, (-5, 0, 0), (0, 0, 0.12)),
    (12, (10, 0, 0), (0, 0, 0.02)),
    (15, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(jump, "L_Leg", [(1, (25, 0, 0)), (4, (-20, 0, 0)), (8, (-10, 0, 0)), (15, (0, 0, 0))])
set_bone_keys(jump, "R_Leg", [(1, (25, 0, 0)), (4, (-20, 0, 0)), (8, (-10, 0, 0)), (15, (0, 0, 0))])
set_bone_keys(jump, "L_Arm", [(1, (20, 0, 20)), (4, (-40, 0, 40)), (15, (8, 0, 12))])
set_bone_keys(jump, "R_Arm", [(1, (20, 0, -20)), (4, (-40, 0, -40)), (15, (8, 0, -12))])
set_bone_keys(jump, "Head", [(1, (10, 0, 0)), (4, (-15, 0, 0)), (15, (0, 0, 0))])

# emote_wave — one-shot ~1.2s = 30 frames
clear_pose()
wave = ensure_action("emote_wave")
set_bone_keys(wave, "Spine", [(1, (0, 0, 0)), (8, (0, 0, -12)), (30, (0, 0, 0))])
set_bone_keys(wave, "R_Arm", [
    (1, (8, 0, -12)),
    (6, (-110, 0, -20)),
    (12, (-110, 0, 25)),
    (18, (-110, 0, -25)),
    (24, (-110, 0, 20)),
    (30, (8, 0, -12)),
])
set_bone_keys(wave, "R_Forearm", [
    (1, (0, 0, 0)),
    (6, (-20, 0, 0)),
    (12, (10, 0, 0)),
    (18, (-20, 0, 0)),
    (24, (10, 0, 0)),
    (30, (0, 0, 0)),
])
set_bone_keys(wave, "Head", [(1, (0, 0, 0)), (8, (0, 0, 15)), (30, (0, 0, 0))])

# emote_dance — looping party bounce ~1s = 24 frames
clear_pose()
dance = ensure_action("emote_dance")
set_bone_keys(dance, "Hips", [
    (1, (0, 0, 0), (0, 0, 0.0)),
    (6, (0, 12, 0), (0, 0, 0.04)),
    (12, (0, 0, 0), (0, 0, 0.0)),
    (18, (0, -12, 0), (0, 0, 0.04)),
    (24, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(dance, "Spine", [
    (1, (0, 0, -10)), (12, (0, 0, 10)), (24, (0, 0, -10)),
])
set_bone_keys(dance, "Head", [
    (1, (0, 0, 8)), (12, (0, 0, -8)), (24, (0, 0, 8)),
])
set_bone_keys(dance, "L_Arm", [
    (1, (-80, 0, 40)), (12, (-60, 0, 60)), (24, (-80, 0, 40)),
])
set_bone_keys(dance, "R_Arm", [
    (1, (-60, 0, -60)), (12, (-80, 0, -40)), (24, (-60, 0, -60)),
])
set_bone_keys(dance, "L_Leg", [(1, (15, 0, 10)), (12, (15, 0, -10)), (24, (15, 0, 10))])
set_bone_keys(dance, "R_Leg", [(1, (15, 0, -10)), (12, (15, 0, 10)), (24, (15, 0, -10))])

# Push actions into NLA strips so glTF exports all clips (Blender 4+/5)
clear_pose()
arm_obj.animation_data_create()
track_i = 0
clip_meta = [
    ("idle", idle, 1, 48),
    ("walk", walk, 1, 20),
    ("run", run, 1, 12),
    ("jump", jump, 1, 15),
    ("emote_wave", wave, 1, 30),
    ("emote_dance", dance, 1, 24),
]
for name, action, f0, f1 in clip_meta:
    track = arm_obj.animation_data.nla_tracks.new()
    track.name = name
    strip = track.strips.new(name, int(f0), action)
    strip.action_frame_start = f0
    strip.action_frame_end = f1
    strip.frame_start = f0
    strip.frame_end = f1
    track_i += 1

# Prefer active action cleared so NLA drives export
arm_obj.animation_data.action = None

OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
export_kwargs = dict(
    filepath=str(OUT_PATH),
    export_format="GLB",
    use_selection=False,
    export_apply=False,
    export_texcoords=True,
    export_normals=True,
    export_materials="EXPORT",
    export_image_format="JPEG",
    export_yup=True,
    export_skins=True,
    export_animations=True,
    export_nla_strips=True,
    export_def_bones=False,
    export_optimize_animation_size=True,
)
try:
    bpy.ops.export_scene.gltf(**export_kwargs, export_jpeg_quality=85)
except TypeError:
    # Older/newer kw variants
    for drop in ("export_nla_strips", "export_optimize_animation_size", "export_def_bones"):
        export_kwargs.pop(drop, None)
    bpy.ops.export_scene.gltf(**export_kwargs)

print("RIG_OK", ASSET_ID)
print("faces", len(body.data.polygons))
print("bones", [b.name for b in arm_data.bones])
print("clips", [t.name for t in arm_obj.animation_data.nla_tracks])
print("bytes", OUT_PATH.stat().st_size)
'''


def _gltf_transform(*args: str) -> None:
    npx = shutil.which("npx") or shutil.which("npx.cmd")
    if not npx:
        raise RuntimeError("npx not found on PATH")
    cmd = [npx, "--yes", "@gltf-transform/cli@4.1.1", *args]
    print("+", " ".join(cmd))
    proc = subprocess.run(cmd, capture_output=True, text=True, shell=False)
    if proc.stdout:
        print(proc.stdout[-1500:])
    if proc.returncode != 0:
        print(proc.stderr[-2000:], file=sys.stderr)
        raise RuntimeError(f"gltf-transform failed: {' '.join(args[:2])}")


def _simplify(src: Path, dst: Path, *, ratio: float, error: float) -> None:
    weld = dst.with_name(dst.stem + "_weld.glb")
    try:
        _gltf_transform("weld", str(src), str(weld))
        _gltf_transform(
            "simplify",
            str(weld),
            str(dst),
            "--ratio",
            str(ratio),
            "--error",
            str(error),
            "--lock-border",
            "true",
        )
    finally:
        weld.unlink(missing_ok=True)


def _register_notes(asset_id: str) -> None:
    if not _REGISTRY.is_file():
        return
    registry = json.loads(_REGISTRY.read_text(encoding="utf-8"))
    for entry in registry.get("assets", []):
        if entry.get("asset_id") == asset_id:
            entry["notes"] = (
                "Skinned Pudgy contract + clips "
                f"({', '.join(_CLIP_NAMES)}); game-res ~100k; Bevy-safe JPEG."
            )
            break
    _REGISTRY.write_text(json.dumps(registry, indent=2) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--asset-id", required=True)
    parser.add_argument(
        "--src",
        type=Path,
        default=None,
        help="Source GLB (default: assets/models/<id>/<id>.glb)",
    )
    parser.add_argument("--simplify-ratio", type=float, default=0.35,
                        help="Vertex keep ratio before skinning (0 skips). "
                             "0.35 keeps more detail and avoids holey meshopt collapses.")
    parser.add_argument("--simplify-error", type=float, default=0.05)
    parser.add_argument(
        "--skip-simplify",
        action="store_true",
        help="Rig the current GLB without a pre-simplify pass",
    )
    args = parser.parse_args()

    if not _BLENDER.is_file():
        print("error: Blender not found", file=sys.stderr)
        return 1
    if shutil.which("npx") is None and not args.skip_simplify:
        print("error: npx required for UV-aware simplify", file=sys.stderr)
        return 1

    aid = args.asset_id.strip()
    dest_dir = _MODELS / aid
    out = dest_dir / f"{aid}.glb"
    src = args.src if args.src is not None else out
    if not src.is_file():
        print(f"error: missing {src}", file=sys.stderr)
        return 1

    dest_dir.mkdir(parents=True, exist_ok=True)
    work_src = dest_dir / f"{aid}_rig_src.glb"
    if args.skip_simplify:
        shutil.copy2(src, work_src)
    else:
        print(f"simplify {src} -> {work_src} (ratio={args.simplify_ratio})")
        _simplify(src, work_src, ratio=args.simplify_ratio, error=args.simplify_error)

    worker = dest_dir / "_rig_worker.py"
    script = (
        _WORKER.replace("__IN_PATH__", str(work_src.resolve()).replace("\\", "/"))
        .replace("__OUT_PATH__", str(out.resolve()).replace("\\", "/"))
        .replace("__ASSET_ID__", aid)
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
        work_src.unlink(missing_ok=True)

    print(proc.stdout[-5000:] if proc.stdout else "")
    if proc.stderr:
        err_tail = proc.stderr[-2000:]
        if "Error" in err_tail or "Traceback" in err_tail:
            print(err_tail, file=sys.stderr)
    if proc.returncode != 0 or not out.is_file() or "RIG_OK" not in (proc.stdout or ""):
        print(proc.stderr[-5000:] if proc.stderr else "", file=sys.stderr)
        print("error: Blender rig export did not complete", file=sys.stderr)
        return 1

    (dest_dir / "README.txt").write_text(
        f"{aid}\n"
        "Skinned Pudgy character (shared armature contract).\n"
        f"Clips: {', '.join(_CLIP_NAMES)}\n"
        "Bones: Root, Hips, Spine, Head, L_Arm, R_Arm, L_Forearm, R_Forearm, "
        "L_Leg, R_Leg, L_Shin, R_Shin\n"
        "Sockets parented to bones. Bevy-safe (no Draco/WebP).\n"
        "Rebuild: python scripts/rig_and_animate_pudgy.py --asset-id "
        f"{aid}\n",
        encoding="utf-8",
    )
    _register_notes(aid)
    print(f"glb -> {out.relative_to(_REPO)} ({out.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
