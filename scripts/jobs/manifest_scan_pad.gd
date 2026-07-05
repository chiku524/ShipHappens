extends Area3D

## Scans manifest crates pushed onto the pad.

@onready var label: Label3D = $Label3D


func _ready() -> void:
	body_entered.connect(_on_body_entered)
	label.text = "SCAN PAD\nPush crate here"


func _on_body_entered(body: Node3D) -> void:
	if not JobSystem.manifest_active or JobSystem.manifest_complete:
		return
	if not body.is_in_group("manifest_crate"):
		return
	if not multiplayer.is_server():
		return
	if body.has_method("mark_scanned") and body.mark_scanned():
		JobSystem.scan_manifest_crate()
		label.text = "SCAN PAD\n%d/%d" % [JobSystem.manifest_scanned, JobSystem.MANIFEST_CRATES_REQUIRED]
