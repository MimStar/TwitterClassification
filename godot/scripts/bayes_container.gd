extends Control

var filedialog
var database_path = ""
var tweet = ""
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

func _on_modes_button_item_selected(index: int) -> void:
	mode = index
	pass # Replace with function body.

func _on_tweet_edit_text_changed() -> void:
	tweet = $TweetEdit.text
	check_to_enable_or_disable_launch_button()
	pass # Replace with function body.
	
func check_to_enable_or_disable_launch_button():
	if tweet.is_empty():
		$LaunchButton.disabled = true
	else:
		$LaunchButton.disabled = false
		
	if database_path.is_empty():
		$EvaluateButton.disabled = true
	else:
		$EvaluateButton.disabled = false
	pass

func _on_launch_button_button_up() -> void:
	var bayes_node = Bayes.new()
	var classe = bayes_node.bayes_execute(database_path,tweet,mode)
	$ResultLabel.text = classe
	pass # Replace with function body.

func _on_evaluate_button_button_up() -> void:
	var bayes_node = Bayes.new()
	var tableau_string = bayes_node.bayes_evaluate(database_path, mode)
	$EvaluationWindow/EvaluationTableLabel.text = tableau_string
	$EvaluationWindow.visible = true
	pass # Replace with function body.

func _on_evaluation_window_close_requested() -> void:
	$EvaluationWindow.visible = false
	pass # Replace with function body.
