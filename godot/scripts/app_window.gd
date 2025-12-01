extends Control

const csv_editor_scene = preload("res://scenes/csv_container.tscn")

var csv_path
var filedialog

func clean_main_window():
	for child in $MainWindow.get_children():
		child.queue_free()
	pass

func _on_import_button_button_up() -> void:
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a csv file that contains tweets"
	filedialog.file_selected.connect(_on_clean_csv_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass # Replace with function body.

func _on_clean_csv_file_selected(path):
	clean_main_window()
	filedialog.hide()
	$LogLabel.text = "Cleaning a file..."
	$ButtonList/ImportButton.disabled = true
	$ButtonList/EditorButton.disabled = true
	$ButtonList/AlgoButton.disabled = true
	await get_tree().create_timer(0.1).timeout
	var clean_data = CleanData.new()
	csv_path = clean_data.clean_data(path)
	$LogLabel.text = "File cleaned"
	var editor_container = csv_editor_scene.instantiate()
	editor_container.csv_path = csv_path
	editor_container.csv_success.connect(_on_csv_success)
	editor_container.csv_error.connect(_on_csv_error)
	$MainWindow.add_child(editor_container)
	$ButtonList/ImportButton.disabled = false
	$ButtonList/AlgoButton.disabled = false
	pass # Replace with function body.

func _on_editor_button_button_up() -> void:
	clean_main_window()
	$ButtonList/EditorButton.disabled = true
	$ButtonList/AlgoButton.disabled = false
	var editor_container = csv_editor_scene.instantiate()
	editor_container.csv_path = csv_path
	editor_container.csv_success.connect(_on_csv_success)
	editor_container.csv_error.connect(_on_csv_error)
	pass # Replace with function body.

func _on_csv_success():
	$LogLabel.text = "File saved in " + csv_path
	pass
	
func _on_csv_error():
	$LogLabel.text = "Error while saving csv file."
	pass

func _on_algo_button_button_up() -> void:
	clean_main_window()
	$ButtonList/EditorButton.disabled = false
	$ButtonList/AlgoButton.disabled = true
	
	pass # Replace with function body.
