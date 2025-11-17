extends Control

var filedialog
var database_path = ""
var k = 3
var mode = 0

func _on_import_database_button_up() -> void:
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a csv with tweets cleaned up to train the model"
	filedialog.file_selected.connect(_on_knn_csv_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass # Replace with function body.

func _on_knn_csv_file_selected(path):
	database_path = path
	check_to_enable_or_disable_launch_button()
	pass

func _on_spin_box_value_changed(value: float) -> void:
	k = int(value)
	pass # Replace with function body.

func _on_modes_button_item_selected(index: int) -> void:
	mode = index
	pass # Replace with function body.
	
func check_to_enable_or_disable_launch_button():
	if database_path.is_empty():
		$LaunchButton.disabled = true
		$EvaluateButton.disabled = true
	else:
		$LaunchButton.disabled = false
		$EvaluateButton.disabled = false
	pass

func _on_launch_button_button_up() -> void:
	var clustering_node = Clustering.new()
	var visualization = clustering_node.hierarchical_execute(database_path,k,mode)
	$DendrogramLabel.text = visualization
	pass # Replace with function body.

func _on_evaluate_button_button_up() -> void:
	var knn_node = Knn.new()
	var tableau_string = knn_node.knn_evaluate(database_path,k,mode)
	$EvaluationWindow/EvaluationTableLabel.text = tableau_string
	$EvaluationWindow.visible = true
	pass # Replace with function body.

func _on_evaluation_window_close_requested() -> void:
	$EvaluationWindow.visible = false
	pass # Replace with function body.
