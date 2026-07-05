extends Control

## Satisfaction meter and job board HUD for ShipHappens.

@onready var satisfaction_bar: ProgressBar = $MarginContainer/HBoxContainer/LeftColumn/SatisfactionBar
@onready var satisfaction_label: Label = $MarginContainer/HBoxContainer/LeftColumn/SatisfactionLabel
@onready var jobs_label: Label = $MarginContainer/HBoxContainer/LeftColumn/JobsLabel
@onready var job_board: RichTextLabel = $MarginContainer/HBoxContainer/JobBoardPanel/MarginContainer/JobBoard
@onready var objective_label: Label = $MarginContainer/HBoxContainer/JobBoardPanel/MarginContainer/ObjectiveLabel


func _ready() -> void:
	JobSystem.satisfaction_changed.connect(_on_satisfaction_changed)
	JobSystem.job_board_changed.connect(_on_job_board_changed)
	GameState.satisfaction_changed.connect(_on_satisfaction_changed)
	GameState.jobs_progress_changed.connect(_on_jobs_progress_changed)
	_refresh_all()


func _refresh_all() -> void:
	_on_satisfaction_changed(GameState.corporate_satisfaction)
	_on_jobs_progress_changed(GameState.jobs_completed, GameState.jobs_required)
	_on_job_board_changed(JobSystem.get_active_jobs(), JobSystem.get_board_progress_text())


func _on_satisfaction_changed(value: float) -> void:
	satisfaction_bar.value = value
	satisfaction_label.text = "Corporate Satisfaction: %d%%" % int(value)


func _on_jobs_progress_changed(completed: int, required: int) -> void:
	jobs_label.text = "Jobs: %d / %d" % [completed, required]


func _on_job_board_changed(active_jobs: Array, progress_text: String) -> void:
	var lines: PackedStringArray = ["[b]Job Board[/b]", ""]
	if active_jobs.is_empty():
		lines.append("No active jobs.")
	else:
		for job in active_jobs:
			lines.append("• %s" % job.get("name", "Unknown"))
			if job.has("progress"):
				lines.append("  %s" % job.get("progress"))
	job_board.text = "\n".join(lines)
	objective_label.text = progress_text
