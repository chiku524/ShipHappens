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

# --- Shared Pudgy armature (stubby mascot proportions) ---
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

# Z-up floor pivot (matches imported polish). Short flipper/foot bones stay near surface.
root = add_bone("Root", (cx, cy, 0.0), (cx, cy, h * 0.06))
hips = add_bone("Hips", (cx, cy, h * 0.32), (cx, cy, h * 0.42), root)
spine = add_bone("Spine", (cx, cy, h * 0.42), (cx, cy, h * 0.58), hips)
head = add_bone("Head", (cx, cy, h * 0.58), (cx, cy, h * 0.88), spine)

# Flipper stubs — short, out to the sides (not long humanoid arms).
l_arm = add_bone("L_Arm", (cx + w * 0.28, cy, h * 0.52), (cx + w * 0.40, cy, h * 0.48), spine)
l_fore = add_bone("L_Forearm", (cx + w * 0.40, cy, h * 0.48), (cx + w * 0.46, cy, h * 0.44), l_arm)
r_arm = add_bone("R_Arm", (cx - w * 0.28, cy, h * 0.52), (cx - w * 0.40, cy, h * 0.48), spine)
r_fore = add_bone("R_Forearm", (cx - w * 0.40, cy, h * 0.48), (cx - w * 0.46, cy, h * 0.44), r_arm)

# Tiny feet under the dumpling.
l_leg = add_bone("L_Leg", (cx + w * 0.12, cy, h * 0.22), (cx + w * 0.14, cy, h * 0.10), hips)
l_shin = add_bone("L_Shin", (cx + w * 0.14, cy, h * 0.10), (cx + w * 0.14, cy, 0.02), l_leg)
r_leg = add_bone("R_Leg", (cx - w * 0.12, cy, h * 0.22), (cx - w * 0.14, cy, h * 0.10), hips)
r_shin = add_bone("R_Shin", (cx - w * 0.14, cy, h * 0.10), (cx - w * 0.14, cy, 0.02), r_leg)

bpy.ops.object.mode_set(mode="OBJECT")

# Prefer envelope weights tuned for blobs (AUTO heat fails on Tripo density).
bpy.ops.object.select_all(action="DESELECT")
body.select_set(True)
arm_obj.select_set(True)
bpy.context.view_layer.objects.active = arm_obj

# Tight limb envelopes so torso stays on Hips/Spine.
ENVELOPE = {
    "Root": (0.45, 0.18, 0.14),
    "Hips": (0.55, 0.22, 0.18),
    "Spine": (0.42, 0.18, 0.16),
    "Head": (0.32, 0.16, 0.14),
    "L_Arm": (0.10, 0.07, 0.06),
    "R_Arm": (0.10, 0.07, 0.06),
    "L_Forearm": (0.08, 0.06, 0.05),
    "R_Forearm": (0.08, 0.06, 0.05),
    "L_Leg": (0.10, 0.07, 0.06),
    "R_Leg": (0.10, 0.07, 0.06),
    "L_Shin": (0.08, 0.06, 0.05),
    "R_Shin": (0.08, 0.06, 0.05),
}
bpy.ops.object.mode_set(mode="POSE")
for pb in arm_obj.pose.bones:
    dist, head_r, tail_r = ENVELOPE.get(pb.name, (0.2, 0.1, 0.08))
    pb.bone.envelope_distance = dist
    pb.bone.head_radius = head_r
    pb.bone.tail_radius = tail_r
bpy.ops.object.mode_set(mode="OBJECT")
bpy.ops.object.parent_set(type="ARMATURE_ENVELOPE")

def count_unweighted():
    bad = []
    for vert in body.data.vertices:
        total = sum(g.weight for g in vert.groups)
        if total < 1e-4:
            bad.append(vert.index)
    return bad

unweighted = count_unweighted()
print("weight groups", len(body.vertex_groups), "unweighted", len(unweighted))

# Pull limb influence off the dumpling core so walk flaps don't melt the belly.
LIMB_GROUPS = ("L_Arm", "R_Arm", "L_Forearm", "R_Forearm", "L_Leg", "R_Leg", "L_Shin", "R_Shin")
core_r2 = (w * 0.28) ** 2
core_z0, core_z1 = h * 0.28, h * 0.72
hips_vg = body.vertex_groups.get("Hips") or body.vertex_groups.new(name="Hips")
spine_vg = body.vertex_groups.get("Spine") or body.vertex_groups.new(name="Spine")
limb_vgs = [body.vertex_groups.get(n) for n in LIMB_GROUPS if body.vertex_groups.get(n)]
stripped = 0
mw = body.matrix_world
for vert in body.data.vertices:
    wp = mw @ vert.co
    dx, dy = wp.x - cx, wp.y - cy
    in_core = (dx * dx + dy * dy) <= core_r2 and core_z0 <= wp.z <= core_z1
    if not in_core:
        continue
    limb_w = 0.0
    for g in vert.groups:
        vg = body.vertex_groups[g.group]
        if vg.name in LIMB_GROUPS:
            limb_w += g.weight
    if limb_w < 0.08:
        continue
    for vg in limb_vgs:
        try:
            vg.add([vert.index], 0.0, "REPLACE")
        except RuntimeError:
            pass
    # Re-home to torso.
    spine_vg.add([vert.index], 0.55, "ADD")
    hips_vg.add([vert.index], 0.45, "ADD")
    stripped += 1
print("CORE_STRIP_LIMBS", stripped)

# Normalize weights per vertex (mesh must be active).
bpy.ops.object.select_all(action="DESELECT")
body.select_set(True)
bpy.context.view_layer.objects.active = body
bpy.ops.object.vertex_group_normalize_all(lock_active=False)

unweighted = count_unweighted()
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

# --- Soft mascot clip authoring (hip bounce + tiny flaps) ---
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

# idle — soft breathe
clear_pose()
idle = ensure_action("idle")
set_bone_keys(idle, "Hips", [
    (1, (0, 0, 0), (0, 0, 0)),
    (24, (1.5, 0, 0), (0, 0, 0.01)),
    (48, (0, 0, 0), (0, 0, 0)),
])
set_bone_keys(idle, "Spine", [(1, (0, 0, 0)), (24, (-2, 0, 0)), (48, (0, 0, 0))])
set_bone_keys(idle, "Head", [(1, (0, 0, 0)), (24, (2, 0, 2)), (48, (0, 0, 0))])
set_bone_keys(idle, "L_Arm", [(1, (4, 0, 6)), (24, (6, 0, 8)), (48, (4, 0, 6))])
set_bone_keys(idle, "R_Arm", [(1, (4, 0, -6)), (24, (6, 0, -8)), (48, (4, 0, -6))])

# walk — dumpling bounce + tiny flaps (~0.9s)
clear_pose()
walk = ensure_action("walk")
set_bone_keys(walk, "Hips", [
    (1, (0, 0, 0), (0, 0, 0.0)),
    (6, (0, 3, 0), (0, 0, 0.018)),
    (11, (0, 0, 0), (0, 0, 0.0)),
    (16, (0, -3, 0), (0, 0, 0.018)),
    (22, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(walk, "Spine", [(1, (2, 0, 0)), (11, (3, 0, 0)), (22, (2, 0, 0))])
set_bone_keys(walk, "Head", [(1, (0, 0, -2)), (11, (0, 0, 2)), (22, (0, 0, -2))])
set_bone_keys(walk, "L_Leg", [(1, (-8, 0, 0)), (11, (8, 0, 0)), (22, (-8, 0, 0))])
set_bone_keys(walk, "R_Leg", [(1, (8, 0, 0)), (11, (-8, 0, 0)), (22, (8, 0, 0))])
set_bone_keys(walk, "L_Shin", [(1, (4, 0, 0)), (11, (2, 0, 0)), (22, (4, 0, 0))])
set_bone_keys(walk, "R_Shin", [(1, (2, 0, 0)), (11, (4, 0, 0)), (22, (2, 0, 0))])
set_bone_keys(walk, "L_Arm", [(1, (6, 0, 10)), (11, (-4, 0, 8)), (22, (6, 0, 10))])
set_bone_keys(walk, "R_Arm", [(1, (-4, 0, -8)), (11, (6, 0, -10)), (22, (-4, 0, -8))])

# run — faster bounce, still soft flaps
clear_pose()
run = ensure_action("run")
set_bone_keys(run, "Hips", [
    (1, (4, 0, 0), (0, 0, 0.02)),
    (4, (4, 5, 0), (0, 0, 0.035)),
    (7, (4, 0, 0), (0, 0, 0.02)),
    (10, (4, -5, 0), (0, 0, 0.035)),
    (13, (4, 0, 0), (0, 0, 0.02)),
])
set_bone_keys(run, "Spine", [(1, (5, 0, 0)), (7, (6, 0, 0)), (13, (5, 0, 0))])
set_bone_keys(run, "L_Leg", [(1, (-12, 0, 0)), (7, (12, 0, 0)), (13, (-12, 0, 0))])
set_bone_keys(run, "R_Leg", [(1, (12, 0, 0)), (7, (-12, 0, 0)), (13, (12, 0, 0))])
set_bone_keys(run, "L_Arm", [(1, (10, 0, 14)), (7, (-8, 0, 10)), (13, (10, 0, 14))])
set_bone_keys(run, "R_Arm", [(1, (-8, 0, -10)), (7, (10, 0, -14)), (13, (-8, 0, -10))])
set_bone_keys(run, "Head", [(1, (0, 0, -3)), (7, (0, 0, 3)), (13, (0, 0, -3))])

# jump — squash / stretch on torso
clear_pose()
jump = ensure_action("jump")
set_bone_keys(jump, "Hips", [
    (1, (8, 0, 0), (0, 0, -0.03)),
    (4, (-6, 0, 0), (0, 0, 0.06)),
    (8, (-3, 0, 0), (0, 0, 0.09)),
    (12, (4, 0, 0), (0, 0, 0.015)),
    (15, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(jump, "Spine", [(1, (6, 0, 0)), (4, (-4, 0, 0)), (15, (0, 0, 0))])
set_bone_keys(jump, "L_Arm", [(1, (8, 0, 10)), (4, (-15, 0, 18)), (15, (4, 0, 6))])
set_bone_keys(jump, "R_Arm", [(1, (8, 0, -10)), (4, (-15, 0, -18)), (15, (4, 0, -6))])
set_bone_keys(jump, "L_Leg", [(1, (10, 0, 0)), (4, (-8, 0, 0)), (15, (0, 0, 0))])
set_bone_keys(jump, "R_Leg", [(1, (10, 0, 0)), (4, (-8, 0, 0)), (15, (0, 0, 0))])
set_bone_keys(jump, "Head", [(1, (6, 0, 0)), (4, (-8, 0, 0)), (15, (0, 0, 0))])

# emote_wave — small flap, body mostly still
clear_pose()
wave = ensure_action("emote_wave")
set_bone_keys(wave, "Spine", [(1, (0, 0, 0)), (8, (0, 0, -6)), (30, (0, 0, 0))])
set_bone_keys(wave, "R_Arm", [
    (1, (4, 0, -6)),
    (6, (-55, 0, -10)),
    (12, (-55, 0, 12)),
    (18, (-55, 0, -12)),
    (24, (-55, 0, 10)),
    (30, (4, 0, -6)),
])
set_bone_keys(wave, "R_Forearm", [
    (1, (0, 0, 0)), (6, (-8, 0, 0)), (12, (6, 0, 0)), (18, (-8, 0, 0)), (24, (6, 0, 0)), (30, (0, 0, 0)),
])
set_bone_keys(wave, "Head", [(1, (0, 0, 0)), (8, (0, 0, 8)), (30, (0, 0, 0))])

# emote_dance — happy hip wiggle
clear_pose()
dance = ensure_action("emote_dance")
set_bone_keys(dance, "Hips", [
    (1, (0, 0, 0), (0, 0, 0.0)),
    (6, (0, 6, 0), (0, 0, 0.025)),
    (12, (0, 0, 0), (0, 0, 0.0)),
    (18, (0, -6, 0), (0, 0, 0.025)),
    (24, (0, 0, 0), (0, 0, 0.0)),
])
set_bone_keys(dance, "Spine", [(1, (0, 0, -5)), (12, (0, 0, 5)), (24, (0, 0, -5))])
set_bone_keys(dance, "Head", [(1, (0, 0, 4)), (12, (0, 0, -4)), (24, (0, 0, 4))])
set_bone_keys(dance, "L_Arm", [(1, (-25, 0, 18)), (12, (-18, 0, 28)), (24, (-25, 0, 18))])
set_bone_keys(dance, "R_Arm", [(1, (-18, 0, -28)), (12, (-25, 0, -18)), (24, (-18, 0, -28))])
set_bone_keys(dance, "L_Leg", [(1, (6, 0, 4)), (12, (6, 0, -4)), (24, (6, 0, 4))])
set_bone_keys(dance, "R_Leg", [(1, (6, 0, -4)), (12, (6, 0, 4)), (24, (6, 0, -4))])

# Push actions into NLA strips so glTF exports all clips (Blender 4+/5)
clear_pose()
arm_obj.animation_data_create()
clip_meta = [
    ("idle", idle, 1, 48),
    ("walk", walk, 1, 22),
    ("run", run, 1, 13),
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
