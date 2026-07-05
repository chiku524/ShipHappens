class_name Interactable
extends Area3D

## Base class for E-key interactables in ShipHappens.

@export var prompt_text: String = "Interact"


func _ready() -> void:
	add_to_group("interactable")


func get_prompt(_player: Node3D) -> String:
	return prompt_text


func can_interact(_player: Node3D) -> bool:
	return true


func interact(_player: Node3D) -> void:
	pass
