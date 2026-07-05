extends Interactable

## Slime puddle cleaned with an equipped mop.

@onready var mesh: MeshInstance3D = $MeshInstance3D
var cleaned: bool = false


func _ready() -> void:
	super._ready()
	collision_layer = 8
	prompt_text = "Mop puddle"


func get_prompt(_player: Node3D) -> String:
	if cleaned or JobSystem.mop_complete:
		return ""
	return "Mop puddle"


func can_interact(player: Node3D) -> bool:
	return (
		not cleaned
		and JobSystem.mop_active
		and not JobSystem.mop_complete
		and player.has_method("has_mop_equipped")
		and player.has_mop_equipped()
	)


func interact(_player: Node3D) -> void:
	if not can_interact(_player):
		return
	if multiplayer.is_server():
		_clean()
	else:
		_request_clean.rpc_id(1, name)


func _clean() -> void:
	if cleaned:
		return
	cleaned = true
	visible = false
	collision_layer = 0
	JobSystem.clean_mop_puddle()
	_sync_cleaned.rpc(name)


@rpc("authority", "call_remote", "reliable")
func _sync_cleaned(puddle_name: String) -> void:
	if name != puddle_name:
		return
	cleaned = true
	visible = false
	collision_layer = 0


@rpc("any_peer", "call_remote", "reliable")
func _request_clean(puddle_name: String) -> void:
	if not multiplayer.is_server():
		return
	if name == puddle_name:
		_clean()
