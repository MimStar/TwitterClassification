extends Control

const actions_scene = preload("res://scenes/actions_container.tscn")
const clean_log_scene = preload("res://scenes/clean_log_container.tscn")

var actions_container
var logs_container
var clean_data

func _ready():
	add_actions_container()
	pass

func add_actions_container():
	actions_container = actions_scene.instantiate()
	add_child(actions_container)
	actions_container.get_node("CleanCSVButton").button_up.connect(_on_clean_csv_button_button_up)
	pass

func add_logs_container():
	actions_container.queue_free()
	logs_container = clean_log_scene.instantiate()
	add_child(logs_container)
	pass
	


func _on_clean_csv_button_button_up():
	$FileDialog.popup()
	pass # Replace with function body.

func _on_file_dialog_file_selected(path):
	add_logs_container()
	clean_data = CleanData.new()
	add_child(clean_data)
	clean_data.log_sent.connect(logs_container._on_log_received)
	var new_path = clean_data.clean_data(path)
	pass # Replace with function body.
