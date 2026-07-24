#!/usr/bin/env python3
"""Copy named animation clips from one crew GLB onto another with a matching rig.

Works when source and target share the same bone names (e.g. pink ↔ water Studio
41-bone rig). Does **not** retarget across different hierarchies (Studio 41 ↔
stubby 12) — use `auto_rig_glb.py` / `rig_and_animate_pudgy.py` for that.

Motion sync: after copy, location / euler keyframes are scaled by the target vs
donor mesh AABB ratios (width X, height Z, depth Y) so short/tall/wide bodies
keep proportional bounce and flap amplitude.

Usage:
  # Copy every shared clip from water onto pink (in place)
  python scripts/transfer_crew_clips.py \\
    --from char_pudgy_water_01 \\
    --to char_pudgy_pink_01

  # Donor file → target asset id
  python scripts/transfer_crew_clips.py \\
    --from-glb "C:/Downloads/hero_clips.glb" \\
    --to char_pudgy_forest_01

  # Only specific clips
  python scripts/transfer_crew_clips.py \\
    --from char_pudgy_water_01 --to char_pudgy_pink_01 \\
    --clips idle,walk,run,jump,emote_scared

  # Dry-run bone overlap check
  python scripts/transfer_crew_clips.py --from char_pudgy_water_01 --to char_pudgy_stylized_01 --check
"""

from __future__ import annotations

import argparse
import json
import struct
import subprocess
import sys
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_MODELS = _REPO / "assets" / "models"
_BLENDER = Path(r"C:\Program Files\Blender Foundation\Blender 5.1\blender.exe")

_PARTY_CLIPS = (
    "idle",
    "walk",
    "run",
    "jump",
    "emote_wave",
    "emote_dance",
    "emote_scared",
    "emote_cheer",
)

_WORKER = r'''
import bpy
from pathlib import Path

FROM_PATH = Path(r"__FROM_PATH__")
TO_PATH = Path(r"__TO_PATH__")
OUT_PATH = Path(r"__OUT_PATH__")
CLIP_FILTER = [c for c in "__CLIPS__".split(",") if c]
REPLACE_EXISTING = __REPLACE__

bpy.ops.wm.read_factory_settings(use_empty=True)

def import_gltf(path, prefix):
    before = set(bpy.context.scene.objects)
    bpy.ops.import_scene.gltf(filepath=str(path))
    added = [o for o in bpy.context.scene.objects if o not in before]
    for o in added:
        o.name = f"{prefix}{o.name}"
        if o.data and hasattr(o.data, "name"):
            o.data.name = f"{prefix}{o.data.name}"
    arms = [o for o in added if o.type == "ARMATURE"]
    if not arms:
        raise RuntimeError(f"no armature in {path}")
    return arms[0], added

def joint_names(arm):
    return {b.name for b in arm.data.bones}

def collect_clips(arm):
    """Map clip name -> Action from NLA tracks (preferred) or active action."""
    out = {}
    if arm.animation_data is None:
        return out
    for track in arm.animation_data.nla_tracks:
        name = track.name.strip() or None
        for strip in track.strips:
            if strip.action is None:
                continue
            key = name or strip.action.name
            out[key] = strip.action
            break
    if arm.animation_data.action is not None:
        act = arm.animation_data.action
        out.setdefault(act.name, act)
    return out

def ensure_anim(arm):
    if arm.animation_data is None:
        arm.animation_data_create()
    return arm.animation_data

def remove_track_named(arm, name):
    ad = ensure_anim(arm)
    for track in list(ad.nla_tracks):
        if track.name == name:
            ad.nla_tracks.remove(track)

def push_clip(arm, name, action, loc_scale=(1.0, 1.0, 1.0), rot_scale=(1.0, 1.0, 1.0)):
    ad = ensure_anim(arm)
    remove_track_named(arm, name)
    # Isolate so NLA export picks one strip per track.
    copied = action.copy()
    copied.name = name
    copied.use_fake_user = True
    scale_action_motion(copied, loc_scale, rot_scale)
    track = ad.nla_tracks.new()
    track.name = name
    # Frame range from action
    frame_start = int(copied.frame_range[0]) if copied.frame_range else 1
    frame_end = int(copied.frame_range[1]) if copied.frame_range else max(frame_start + 1, 24)
    strip = track.strips.new(name, max(frame_start, 1), copied)
    strip.name = name
    strip.action = copied
    try:
        strip.action_frame_start = frame_start
        strip.action_frame_end = frame_end
    except Exception:
        pass
    print("PUSH_CLIP", name, f"frames={frame_start}-{frame_end}")

def mesh_aabb_dims(objs):
    """Return (width_x, height_z, depth_y) from mesh world bounds (Blender Z-up)."""
    import mathutils
    minv = mathutils.Vector((1e9, 1e9, 1e9))
    maxv = mathutils.Vector((-1e9, -1e9, -1e9))
    found = False
    for obj in objs:
        if obj.type != "MESH":
            continue
        found = True
        for corner in obj.bound_box:
            wco = obj.matrix_world @ mathutils.Vector(corner)
            minv = mathutils.Vector((min(minv.x, wco.x), min(minv.y, wco.y), min(minv.z, wco.z)))
            maxv = mathutils.Vector((max(maxv.x, wco.x), max(maxv.y, wco.y), max(maxv.z, wco.z)))
    if not found:
        return 1.0, 1.0, 1.0
    return (
        max(maxv.x - minv.x, 1e-4),
        max(maxv.z - minv.z, 1e-4),
        max(maxv.y - minv.y, 1e-4),
    )

def scale_action_motion(action, loc_scale, rot_scale):
    """Scale pose location / euler keyframes to match target mesh XYZ vs donor.

    loc_scale / rot_scale are (x, y, z) multipliers applied by fcurve array_index.
    Pose-bone location: index 0=X lateral, 1=Y along bone (~height), 2=Z depth.
    """
    def _scale_fcurve(fc, scales):
        if fc.array_index < 0 or fc.array_index > 2:
            return
        s = scales[fc.array_index]
        if abs(s - 1.0) < 1e-6:
            return
        for kp in fc.keyframe_points:
            kp.co[1] *= s
            kp.handle_left[1] *= s
            kp.handle_right[1] *= s

    # Blender 4+/5 layered actions expose fcurves via channelsbag / legacy .fcurves.
    fcurves = getattr(action, "fcurves", None)
    if fcurves is None and hasattr(action, "layers"):
        try:
            fcurves = action.layers[0].strips[0].channelbag(action.slots[0]).fcurves
        except Exception:
            fcurves = None
    if not fcurves:
        return
    for fc in fcurves:
        path = fc.data_path or ""
        if path.endswith("location") or ".location" in path:
            _scale_fcurve(fc, loc_scale)
        elif path.endswith("rotation_euler") or ".rotation_euler" in path:
            _scale_fcurve(fc, rot_scale)

src_arm, src_objs = import_gltf(FROM_PATH, "SRC_")
dst_arm, dst_objs = import_gltf(TO_PATH, "DST_")

src_w, src_h, src_d = mesh_aabb_dims(src_objs)
dst_w, dst_h, dst_d = mesh_aabb_dims(dst_objs)

def _ratio(a, b):
    r = a / max(b, 1e-4)
    return max(0.35, min(2.5, r))

# Target / donor — resize donor motion onto the destination body.
SX = _ratio(dst_w, src_w)
SZ = _ratio(dst_h, src_h)
SY = _ratio(dst_d, src_d)
LOC_SCALE = (SX, SZ, SY)  # bone-local X,Y,Z
ROT_SCALE = (
    max(0.45, min(1.8, 0.5 * (SY + SZ))),  # swing
    max(0.45, min(1.8, SX)),                 # twist
    max(0.45, min(1.8, SX)),                 # flare
)
print(
    "DIM_SYNC",
    f"src=({src_w:.3f},{src_h:.3f},{src_d:.3f})",
    f"dst=({dst_w:.3f},{dst_h:.3f},{dst_d:.3f})",
    f"loc_scale=({SX:.3f},{SZ:.3f},{SY:.3f})",
)

src_joints = joint_names(src_arm)
dst_joints = joint_names(dst_arm)
overlap = src_joints & dst_joints
missing = sorted(src_joints - dst_joints)
print("SRC_JOINTS", len(src_joints))
print("DST_JOINTS", len(dst_joints))
print("OVERLAP", len(overlap), f"({100.0 * len(overlap) / max(len(src_joints), 1):.1f}% of source)")
if missing:
    print("MISSING_ON_TARGET", ",".join(missing[:24]), ("..." if len(missing) > 24 else ""))

# Require most source bones so clips don't silently no-op.
ratio = len(overlap) / max(len(src_joints), 1)
if ratio < 0.85:
    raise RuntimeError(
        f"bone overlap too low ({ratio:.0%}) — rigs differ; "
        "use auto_rig_glb.py / rig_and_animate_pudgy.py to retarget"
    )

src_clips = collect_clips(src_arm)
print("SRC_CLIPS", ",".join(sorted(src_clips)))
if not src_clips:
    raise RuntimeError("source GLB has no NLA/action clips to transfer")

wanted = CLIP_FILTER if CLIP_FILTER else sorted(src_clips)
existing = set(collect_clips(dst_arm))
transferred = []
skipped = []
for name in wanted:
    action = src_clips.get(name)
    if action is None:
        print("SKIP_MISSING_SRC", name)
        skipped.append(name)
        continue
    if name in existing and not REPLACE_EXISTING:
        print("KEEP_EXISTING", name)
        skipped.append(name)
        continue
    push_clip(dst_arm, name, action, LOC_SCALE, ROT_SCALE)
    transferred.append(name)

# Clear active action so export uses NLA only.
dst_arm.animation_data.action = None
for track in dst_arm.animation_data.nla_tracks:
    track.mute = False

# Drop source objects before export.
for o in list(src_objs):
    bpy.data.objects.remove(o, do_unlink=True)

# Select destination hierarchy for export.
bpy.ops.object.select_all(action="DESELECT")
for o in bpy.context.scene.objects:
    o.select_set(True)
bpy.context.view_layer.objects.active = dst_arm

OUT_PATH.parent.mkdir(parents=True, exist_ok=True)
export_kwargs = dict(
    filepath=str(OUT_PATH),
    export_format="GLB",
    export_animations=True,
    export_animation_mode="NLA_TRACKS",
    export_nla_strips=True,
    export_def_bones=True,
    export_apply=False,
    export_extras=True,
    export_materials="EXPORT",
    export_cameras=False,
    export_lights=False,
)
try:
    bpy.ops.export_scene.gltf(**export_kwargs)
except TypeError:
    # Older exporter kwds
    for drop in ("export_nla_strips", "export_animation_mode", "export_def_bones"):
        export_kwargs.pop(drop, None)
    bpy.ops.export_scene.gltf(**export_kwargs)

print("TRANSFER_OK", "transferred=", ",".join(transferred), "skipped=", ",".join(skipped))
print("OUT", OUT_PATH, "bytes", OUT_PATH.stat().st_size)
'''


def _asset_glb(asset_id: str) -> Path:
    return _MODELS / asset_id / f"{asset_id}.glb"


def _parse_glb_json(path: Path) -> dict:
    data = path.read_bytes()
    if data[:4] != b"glTF":
        raise RuntimeError(f"not a GLB: {path}")
    offset = 12
    while offset + 8 <= len(data):
        clen = struct.unpack_from("<I", data, offset)[0]
        ctype = struct.unpack_from("<I", data, offset + 4)[0]
        chunk = data[offset + 8 : offset + 8 + clen]
        offset += 8 + clen
        if ctype == 0x4E4F534A:
            return json.loads(chunk.decode("utf-8"))
    raise RuntimeError(f"no JSON chunk in {path}")


def inspect_glb(path: Path) -> dict:
    g = _parse_glb_json(path)
    nodes = g.get("nodes", [])
    skins = g.get("skins", [])
    joints: list[str] = []
    if skins:
        joints = [nodes[i].get("name", f"node_{i}") for i in skins[0].get("joints", [])]
    anims = [a.get("name") or f"anim_{i}" for i, a in enumerate(g.get("animations", []))]
    return {
        "path": str(path),
        "joint_count": len(joints),
        "joints": joints,
        "animations": anims,
    }


def bone_overlap(a: list[str], b: list[str]) -> float:
    sa, sb = set(a), set(b)
    if not sa:
        return 0.0
    return len(sa & sb) / len(sa)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    src = parser.add_mutually_exclusive_group(required=True)
    src.add_argument("--from", dest="from_id", help="Donor asset id under assets/models/")
    src.add_argument("--from-glb", type=Path, help="Donor GLB path")
    dst = parser.add_mutually_exclusive_group(required=True)
    dst.add_argument("--to", dest="to_id", help="Target asset id under assets/models/")
    dst.add_argument("--to-glb", type=Path, help="Target GLB path (overwritten unless --out)")
    parser.add_argument("--out", type=Path, default=None, help="Output GLB (default: overwrite target)")
    parser.add_argument(
        "--clips",
        default="",
        help=f"Comma list to transfer (default: all on donor). Known: {','.join(_PARTY_CLIPS)}",
    )
    parser.add_argument(
        "--keep-existing",
        action="store_true",
        help="Do not overwrite clips that already exist on the target",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Only print bone overlap / clip lists (no Blender write)",
    )
    args = parser.parse_args()

    from_path = _asset_glb(args.from_id) if args.from_id else args.from_glb
    to_path = _asset_glb(args.to_id) if args.to_id else args.to_glb
    if not from_path or not from_path.is_file():
        print(f"error: missing donor {from_path}", file=sys.stderr)
        return 1
    if not to_path or not to_path.is_file():
        print(f"error: missing target {to_path}", file=sys.stderr)
        return 1

    donor = inspect_glb(from_path)
    target = inspect_glb(to_path)
    ratio = bone_overlap(donor["joints"], target["joints"])
    print(f"donor  {from_path.name}: {donor['joint_count']} bones, clips={donor['animations']}")
    print(f"target {to_path.name}: {target['joint_count']} bones, clips={target['animations']}")
    print(f"overlap: {ratio:.0%} of donor bones present on target")

    if args.check:
        ok = ratio >= 0.85
        print("CHECK", "OK" if ok else "FAIL - rigs incompatible for direct transfer")
        return 0 if ok else 2

    if ratio < 0.85:
        print(
            "error: bone overlap too low for direct transfer. "
            "Re-rig the target with auto_rig_glb.py, or use a same-family donor.",
            file=sys.stderr,
        )
        return 2

    if not _BLENDER.is_file():
        print(f"error: Blender not found at {_BLENDER}", file=sys.stderr)
        return 1

    out_path = args.out if args.out is not None else to_path
    work = out_path.parent
    work.mkdir(parents=True, exist_ok=True)
    worker = work / "_transfer_clips_worker.py"
    clips = ",".join(c.strip() for c in args.clips.split(",") if c.strip())
    script = (
        _WORKER.replace("__FROM_PATH__", str(from_path.resolve()).replace("\\", "/"))
        .replace("__TO_PATH__", str(to_path.resolve()).replace("\\", "/"))
        .replace("__OUT_PATH__", str(out_path.resolve()).replace("\\", "/"))
        .replace("__CLIPS__", clips)
        .replace("__REPLACE__", "False" if args.keep_existing else "True")
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

    print(proc.stdout[-6000:] if proc.stdout else "")
    if proc.returncode != 0 or "TRANSFER_OK" not in (proc.stdout or ""):
        print(proc.stderr[-4000:] if proc.stderr else "", file=sys.stderr)
        print("error: clip transfer failed", file=sys.stderr)
        return 1
    print(f"wrote {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
