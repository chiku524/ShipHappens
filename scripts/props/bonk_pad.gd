extends Area3D

## Launches players upward so they can test bonk landings.

@export var launch_velocity := 14.0

@onready var label: Label3D = $Label3D


func _ready() -> void:
	add_to_group("bonk_pad")
	body_entered.connect(_on_body_entered)
	label.text = "BONK PAD\nJump in for chaos"


func _on_body_entered(body: Node3D) -> void:
	if body is CharacterBody3D and body.has_method("apply_launch"):
		body.apply_launch(Vector3.UP * launch_velocity)
