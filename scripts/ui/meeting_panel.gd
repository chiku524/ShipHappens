extends CanvasLayer

## Emergency Stand-Up Meeting vote UI.

@onready var panel: PanelContainer = $Panel
@onready var timer_label: Label = $Panel/MarginContainer/VBoxContainer/TimerLabel
@onready var vote_box: VBoxContainer = $Panel/MarginContainer/VBoxContainer/VoteBox
@onready var status_label: Label = $Panel/MarginContainer/VBoxContainer/StatusLabel

var _voted: bool = false


func _ready() -> void:
	visible = false
	RoundManager.meeting_started.connect(_on_meeting_started)
	RoundManager.meeting_ended.connect(_on_meeting_ended)
	RoundManager.timer_updated.connect(_on_timer_updated)


func _on_meeting_started(_seconds: float) -> void:
	_voted = false
	visible = true
	status_label.text = "Vote to Write Up a suspect, or Skip."
	_build_vote_buttons()


func _on_meeting_ended(written_up_peer_id: int) -> void:
	if written_up_peer_id > 0:
		status_label.text = "Written Up: Player %d" % written_up_peer_id
	else:
		status_label.text = "No Write-Up this time."
	await get_tree().create_timer(2.0).timeout
	visible = false


func _on_timer_updated(_round_seconds: float, _shuttle_seconds: float) -> void:
	if visible:
		timer_label.text = "Meeting: %ds" % maxi(int(RoundManager.meeting_time_remaining), 0)


func _build_vote_buttons() -> void:
	for child in vote_box.get_children():
		child.queue_free()

	var skip_button := Button.new()
	skip_button.text = "Skip (-5% Satisfaction)"
	skip_button.pressed.connect(func(): _cast_vote(0))
	vote_box.add_child(skip_button)

	for node in get_tree().get_nodes_in_group("players"):
		var peer_id := int(node.name)
		var button := Button.new()
		button.text = "Write Up: %s" % node.player_name
		button.pressed.connect(_cast_vote.bind(peer_id))
		vote_box.add_child(button)


func _cast_vote(target_id: int) -> void:
	if _voted:
		return
	_voted = true
	status_label.text = "Vote submitted."
	if multiplayer.is_server():
		RoundManager.submit_vote(multiplayer.get_unique_id(), target_id)
	else:
		_submit_vote.rpc_id(1, target_id)


@rpc("any_peer", "call_remote", "reliable")
func _submit_vote(target_id: int) -> void:
	if multiplayer.is_server():
		RoundManager.submit_vote(multiplayer.get_remote_sender_id(), target_id)
