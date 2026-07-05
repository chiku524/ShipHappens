extends RigidBody3D

## Manifest crate that can only be scanned once.

var scanned: bool = false


func _ready() -> void:
	add_to_group("manifest_crate")
	continuous_cd = true
	if not multiplayer.is_server():
		freeze = true


func mark_scanned() -> bool:
	if scanned:
		return false
	scanned = true
	return true
