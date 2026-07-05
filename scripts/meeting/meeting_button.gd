extends Interactable

## Calls an Emergency Stand-Up Meeting.


func _ready() -> void:
	super._ready()
	collision_layer = 8
	prompt_text = "Call meeting"


func get_prompt(_player: Node3D) -> String:
	if RoundManager.is_meeting_active():
		return "Meeting in progress"
	if RoundManager.meetings_used >= RoundManager.MAX_MEETINGS:
		return "No meetings left"
	return "Call Emergency Meeting"


func can_interact(_player: Node3D) -> bool:
	return (
		not RoundManager.is_meeting_active()
		and RoundManager.meetings_used < RoundManager.MAX_MEETINGS
		and GameState.round_phase == GameState.RoundPhase.PLAYING
	)


func interact(_player: Node3D) -> void:
	if not can_interact(_player):
		return
	if multiplayer.is_server():
		RoundManager.call_meeting(multiplayer.get_unique_id())
	else:
		_request_meeting.rpc_id(1)


@rpc("any_peer", "call_remote", "reliable")
func _request_meeting() -> void:
	if multiplayer.is_server():
		RoundManager.call_meeting(multiplayer.get_remote_sender_id())
