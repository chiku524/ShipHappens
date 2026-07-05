extends Node

## Server-authoritative job tracking for ShipHappens.

signal satisfaction_changed(value: float)
signal job_board_changed(active_jobs: Array, progress_text: String)
signal job_completed(job_id: String)
signal paperwork_state_changed(active: bool, forms_fed: int, complete: bool)

const PAPERWORK_JOB_ID := "paperwork_avalanche"
const PAPERWORK_FORMS_REQUIRED := 5
const PAPERWORK_SATISFACTION := 6.0

var paperwork_active: bool = false
var paperwork_complete: bool = false
var forms_fed: int = 0


func _ready() -> void:
	multiplayer.connected_to_server.connect(_on_connected_to_server)


func reset_jobs() -> void:
	if multiplayer.is_server():
		_reset_local()
		_broadcast_state()
	else:
		_request_reset.rpc_id(1)


func _reset_local() -> void:
	paperwork_active = false
	paperwork_complete = false
	forms_fed = 0
	GameState.reset_round()
	GameState.round_phase = GameState.RoundPhase.PLAYING
	_emit_board()


func start_paperwork_job(requester_id: int) -> bool:
	if not multiplayer.is_server():
		return false
	if paperwork_active or paperwork_complete:
		return false

	paperwork_active = true
	forms_fed = 0
	_broadcast_state()
	return true


func feed_paperwork_form() -> bool:
	if not multiplayer.is_server():
		return false
	if not paperwork_active or forms_fed >= PAPERWORK_FORMS_REQUIRED:
		return false

	forms_fed += 1
	_broadcast_state()
	return true


func complete_paperwork_job() -> bool:
	if not multiplayer.is_server():
		return false
	if not paperwork_active or forms_fed < PAPERWORK_FORMS_REQUIRED:
		return false

	paperwork_active = false
	paperwork_complete = true
	GameState.jobs_completed += 1
	GameState.add_satisfaction(PAPERWORK_SATISFACTION)
	GameState.jobs_progress_changed.emit(GameState.jobs_completed, GameState.jobs_required)
	_broadcast_state()
	job_completed.emit(PAPERWORK_JOB_ID)
	return true


func get_active_jobs() -> Array:
	var jobs: Array = []
	if paperwork_active:
		jobs.append({
			"id": PAPERWORK_JOB_ID,
			"name": "Paperwork Avalanche",
			"progress": "%d/%d forms filed" % [forms_fed, PAPERWORK_FORMS_REQUIRED],
		})
	elif not paperwork_complete:
		jobs.append({
			"id": PAPERWORK_JOB_ID,
			"name": "Paperwork Avalanche",
			"progress": "Start at Job Kiosk",
		})
	return jobs


func get_board_progress_text() -> String:
	if paperwork_complete:
		return "Paperwork Avalanche complete!"
	if paperwork_active:
		if forms_fed >= PAPERWORK_FORMS_REQUIRED:
			return "All forms filed — confirm at Job Kiosk"
		return "Feed forms into the Pneumatic Tube (%d/%d)" % [forms_fed, PAPERWORK_FORMS_REQUIRED]
	return "Accept a job at the Job Kiosk (E)"


func _broadcast_state() -> void:
	_sync_state.rpc(
		paperwork_active,
		paperwork_complete,
		forms_fed,
		GameState.corporate_satisfaction,
		GameState.jobs_completed
	)


func _emit_board() -> void:
	var jobs := get_active_jobs()
	var progress := get_board_progress_text()
	job_board_changed.emit(jobs, progress)
	paperwork_state_changed.emit(paperwork_active, forms_fed, paperwork_complete)
	satisfaction_changed.emit(GameState.corporate_satisfaction)


func _on_connected_to_server() -> void:
	_request_full_state.rpc_id(1)


@rpc("any_peer", "call_remote", "reliable")
func _request_reset() -> void:
	if not multiplayer.is_server():
		return
	_reset_local()
	_broadcast_state()


@rpc("any_peer", "call_remote", "reliable")
func _request_full_state() -> void:
	if not multiplayer.is_server():
		return
	_broadcast_state()


@rpc("authority", "call_remote", "reliable")
func _sync_state(
	active: bool,
	complete: bool,
	fed: int,
	satisfaction: float,
	jobs_done: int
) -> void:
	paperwork_active = active
	paperwork_complete = complete
	forms_fed = fed
	GameState.corporate_satisfaction = satisfaction
	GameState.jobs_completed = jobs_done
	_emit_board()
