extends Node

## Server-authoritative job tracking for ShipHappens.

signal satisfaction_changed(value: float)
signal job_board_changed(active_jobs: Array, progress_text: String)
signal job_completed(job_id: String)
signal paperwork_state_changed(active: bool, forms_fed: int, complete: bool)

const PAPERWORK_JOB_ID := "paperwork_avalanche"
const POWER_HOUR_JOB_ID := "power_hour"
const MOP_JOB_ID := "mop_the_future"
const MANIFEST_JOB_ID := "manifest_lies"

const PAPERWORK_FORMS_REQUIRED := 5
const MOP_PUDDLES_REQUIRED := 6
const MANIFEST_CRATES_REQUIRED := 2

const JOB_SATISFACTION := {
	PAPERWORK_JOB_ID: 6.0,
	POWER_HOUR_JOB_ID: 7.0,
	MOP_JOB_ID: 5.0,
	MANIFEST_JOB_ID: 8.0,
}

const JOB_NAMES := {
	PAPERWORK_JOB_ID: "Paperwork Avalanche",
	POWER_HOUR_JOB_ID: "Power Hour",
	MOP_JOB_ID: "Mop the Future",
	MANIFEST_JOB_ID: "Manifest Lies",
}

var paperwork_active: bool = false
var paperwork_complete: bool = false
var forms_fed: int = 0

var power_hour_active: bool = false
var power_hour_complete: bool = false
var power_hour_step: int = 0
const POWER_HOUR_SEQUENCE := [0, 2, 1, 3]

var mop_active: bool = false
var mop_complete: bool = false
var mop_cleaned: int = 0

var manifest_active: bool = false
var manifest_complete: bool = false
var manifest_scanned: int = 0


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
	power_hour_active = false
	power_hour_complete = false
	power_hour_step = 0
	mop_active = false
	mop_complete = false
	mop_cleaned = 0
	manifest_active = false
	manifest_complete = false
	manifest_scanned = 0
	GameState.jobs_completed = 0
	GameState.corporate_satisfaction = 100.0
	_emit_board()


func is_job_complete(job_id: String) -> bool:
	match job_id:
		PAPERWORK_JOB_ID:
			return paperwork_complete
		POWER_HOUR_JOB_ID:
			return power_hour_complete
		MOP_JOB_ID:
			return mop_complete
		MANIFEST_JOB_ID:
			return manifest_complete
	return false


func start_job(job_id: String) -> bool:
	if not multiplayer.is_server() or is_job_complete(job_id):
		return false
	match job_id:
		PAPERWORK_JOB_ID:
			if paperwork_active:
				return false
			paperwork_active = true
			forms_fed = 0
		POWER_HOUR_JOB_ID:
			if power_hour_active:
				return false
			power_hour_active = true
			power_hour_step = 0
		MOP_JOB_ID:
			if mop_active:
				return false
			mop_active = true
			mop_cleaned = 0
		MANIFEST_JOB_ID:
			if manifest_active:
				return false
			manifest_active = true
			manifest_scanned = 0
		_:
			return false
	_broadcast_state()
	return true


func complete_job(job_id: String) -> bool:
	if not multiplayer.is_server() or is_job_complete(job_id):
		return false
	match job_id:
		PAPERWORK_JOB_ID:
			paperwork_active = false
			paperwork_complete = true
		POWER_HOUR_JOB_ID:
			power_hour_active = false
			power_hour_complete = true
		MOP_JOB_ID:
			mop_active = false
			mop_complete = true
		MANIFEST_JOB_ID:
			manifest_active = false
			manifest_complete = true
		_:
			return false

	GameState.jobs_completed += 1
	GameState.add_satisfaction(JOB_SATISFACTION.get(job_id, 5.0))
	GameState.jobs_progress_changed.emit(GameState.jobs_completed, GameState.jobs_required)
	_broadcast_state()
	job_completed.emit(job_id)
	RoundManager.check_shuttle_unlock()
	return true


func start_paperwork_job(_requester_id: int) -> bool:
	return start_job(PAPERWORK_JOB_ID)


func feed_paperwork_form() -> bool:
	if not multiplayer.is_server() or not paperwork_active:
		return false
	if forms_fed >= PAPERWORK_FORMS_REQUIRED:
		return false
	forms_fed += 1
	_broadcast_state()
	return true


func complete_paperwork_job() -> bool:
	if not paperwork_active or forms_fed < PAPERWORK_FORMS_REQUIRED:
		return false
	return complete_job(PAPERWORK_JOB_ID)


func try_power_hour_breaker(breaker_index: int) -> Dictionary:
	if not multiplayer.is_server() or not power_hour_active or power_hour_complete:
		return {"ok": false}
	if breaker_index == POWER_HOUR_SEQUENCE[power_hour_step]:
		power_hour_step += 1
		if power_hour_step >= POWER_HOUR_SEQUENCE.size():
			complete_job(POWER_HOUR_JOB_ID)
		_broadcast_state()
		return {"ok": true, "done": power_hour_complete}
	_broadcast_state()
	return {"ok": false, "zap": true}


func clean_mop_puddle() -> bool:
	if not multiplayer.is_server() or not mop_active or mop_complete:
		return false
	mop_cleaned += 1
	if mop_cleaned >= MOP_PUDDLES_REQUIRED:
		complete_job(MOP_JOB_ID)
	_broadcast_state()
	return true


func scan_manifest_crate() -> bool:
	if not multiplayer.is_server() or not manifest_active or manifest_complete:
		return false
	manifest_scanned += 1
	if manifest_scanned >= MANIFEST_CRATES_REQUIRED:
		complete_job(MANIFEST_JOB_ID)
	_broadcast_state()
	return true


func get_active_jobs() -> Array:
	var jobs: Array = []
	for job_id in [PAPERWORK_JOB_ID, POWER_HOUR_JOB_ID, MOP_JOB_ID, MANIFEST_JOB_ID]:
		if is_job_complete(job_id):
			continue
		jobs.append({
			"id": job_id,
			"name": JOB_NAMES[job_id],
			"progress": _job_progress_text(job_id),
		})
	return jobs


func get_board_progress_text() -> String:
	if GameState.round_phase == GameState.RoundPhase.EXTRACTION:
		return "Shuttle bay open — get to the yellow ramp!"
	if GameState.round_phase == GameState.RoundPhase.MEETING:
		return "Emergency Stand-Up Meeting in progress."
	if GameState.jobs_completed >= GameState.jobs_required:
		return "All required jobs complete — shuttle is available."
	return "Complete jobs around the hub. Stowaway is smuggling. Call a meeting if suspicious."


func _job_progress_text(job_id: String) -> String:
	match job_id:
		PAPERWORK_JOB_ID:
			if paperwork_active:
				return "%d/%d forms" % [forms_fed, PAPERWORK_FORMS_REQUIRED]
			return "Start at Job Kiosk"
		POWER_HOUR_JOB_ID:
			if power_hour_active:
				return "Breakers %d/%d" % [power_hour_step, POWER_HOUR_SEQUENCE.size()]
			return "Start at Breaker Panel"
		MOP_JOB_ID:
			if mop_active:
				return "%d/%d puddles" % [mop_cleaned, MOP_PUDDLES_REQUIRED]
			return "Start at Mop Closet"
		MANIFEST_JOB_ID:
			if manifest_active:
				return "%d/%d crates scanned" % [manifest_scanned, MANIFEST_CRATES_REQUIRED]
			return "Start at Manifest Terminal"
	return "Available"


func _broadcast_state() -> void:
	_sync_state.rpc(
		paperwork_active,
		paperwork_complete,
		forms_fed,
		power_hour_active,
		power_hour_complete,
		power_hour_step,
		mop_active,
		mop_complete,
		mop_cleaned,
		manifest_active,
		manifest_complete,
		manifest_scanned,
		GameState.corporate_satisfaction,
		GameState.jobs_completed
	)


func _emit_board() -> void:
	job_board_changed.emit(get_active_jobs(), get_board_progress_text())
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
	p_active: bool,
	p_complete: bool,
	p_fed: int,
	ph_active: bool,
	ph_complete: bool,
	ph_step: int,
	m_active: bool,
	m_complete: bool,
	m_cleaned: int,
	man_active: bool,
	man_complete: bool,
	man_scanned: int,
	satisfaction: float,
	jobs_done: int
) -> void:
	paperwork_active = p_active
	paperwork_complete = p_complete
	forms_fed = p_fed
	power_hour_active = ph_active
	power_hour_complete = ph_complete
	power_hour_step = ph_step
	mop_active = m_active
	mop_complete = m_complete
	mop_cleaned = m_cleaned
	manifest_active = man_active
	manifest_complete = man_complete
	manifest_scanned = man_scanned
	GameState.corporate_satisfaction = satisfaction
	GameState.jobs_completed = jobs_done
	_emit_board()
