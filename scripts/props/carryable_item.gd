class_name CarryableItem
extends Interactable

## Lightweight item the player can pick up and carry.

@export var item_id: String = "form"
@export var display_name: String = "Form"

var is_carried: bool = false
var _carrier: Node3D = null


func _ready() -> void:
	super._ready()
	add_to_group("carryable")
	collision_layer = 8
	collision_mask = 2
	body_entered.connect(_on_body_entered)


func get_prompt(_player: Node3D) -> String:
	if is_carried:
		return ""
	return "Pick up %s" % display_name


func can_interact(player: Node3D) -> bool:
	return not is_carried and player.has_method("can_pickup_item") and player.can_pickup_item()


func interact(player: Node3D) -> void:
	if not can_interact(player):
		return
	if player.has_method("pickup_item"):
		player.pickup_item(self)


func pickup_by(player: Node3D, anchor: Node3D) -> void:
	is_carried = true
	_carrier = player
	collision_layer = 0
	collision_mask = 0
	reparent(anchor)
	position = Vector3.ZERO
	rotation = Vector3.ZERO


func drop(at_position: Vector3) -> void:
	is_carried = false
	_carrier = null
	collision_layer = 8
	collision_mask = 2
	var world := get_tree().current_scene
	if world:
		reparent(world)
	global_position = at_position


func consume() -> void:
	queue_free()


func _on_body_entered(body: Node3D) -> void:
	if is_carried:
		return
	if body is CharacterBody3D and body.has_method("register_nearby_carryable"):
		body.register_nearby_carryable(self)
