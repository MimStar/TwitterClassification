extends Control

var main_theme : Theme = preload("res://assets/main_theme.tres")

var tweet = ""
var mode = 0
var k = 3
var representation = 1
var ngram_mode = 2
var database_path = ""
var positive_path = ""
var negative_path = ""

func create_evaluation_window():
	var new_window = Window.new()
	var new_text_label = RichTextLabel.new()
	
	new_window.title = "Evaluation Table"
	new_window.initial_position = Window.WINDOW_INITIAL_POSITION_CENTER_PRIMARY_SCREEN
	new_window.size = Vector2i(640,360)
	new_window.visible = false
	
	new_text_label.bbcode_enabled = true
	new_text_label.scroll_active = false
	new_text_label.shortcut_keys_enabled = false
	new_text_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	new_text_label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	new_text_label.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	new_text_label.theme = main_theme
	
	new_window.add_child(new_text_label)
	new_window.close_requested.connect(_on_close_requested.bind(new_window))
	add_child(new_window)
	return new_window

func _on_close_requested(node):
	node.queue_free()
	pass

func _on_tweet_edit_text_changed() -> void:
	tweet = $TweetEdit.text
	$ResultLabel.text = ""
	if tweet.is_empty():
		$LaunchButtonsContainer/ClassifyButton.disabled = true
	else:
		$LaunchButtonsContainer/ClassifyButton.disabled = false
	pass # Replace with function body.


func _on_algo_button_item_selected(index: int) -> void:
	$ResultLabel.text = ""
	match index:
		0:
			if positive_path.is_empty() or negative_path.is_empty():
				$LaunchButtonsContainer/ClassifyButton.disabled = true
				$LaunchButtonsContainer/EvaluateButton.disabled = true
			elif tweet.is_empty():
				$LaunchButtonsContainer/ClassifyButton.disabled = true
			else:
				$LaunchButtonsContainer/ClassifyButton.disabled = false
				$LaunchButtonsContainer/EvaluateButton.disabled = false
			$NaiveOptionsBar.show()
			$KNNOptionsBar.hide()
			$BayesOptionsBar.hide()
			$ClusterOptionsBar.hide()
			k = $NaiveOptionsBar/Poids/SpinBox.value
		1:
			if tweet.is_empty():
				$LaunchButtonsContainer/ClassifyButton.disabled = true
			else:
				$LaunchButtonsContainer/ClassifyButton.disabled = false
			$NaiveOptionsBar.hide()
			$KNNOptionsBar.show()
			$BayesOptionsBar.hide()
			$ClusterOptionsBar.hide()
			k = $KNNOptionsBar/KVoisinsMargin/KVoisins/SpinBox.value
			mode = $KNNOptionsBar/ModesButton.selected
		2:
			if tweet.is_empty():
				$LaunchButtonsContainer/ClassifyButton.disabled = true
			else:
				$LaunchButtonsContainer/ClassifyButton.disabled = false
			$NaiveOptionsBar.hide()
			$KNNOptionsBar.hide()
			$ClusterOptionsBar.show()
			$BayesOptionsBar.hide()
			k = $ClusterOptionsBar/KVoisinsMargin/KClusters/SpinBox.value
			mode = $ClusterOptionsBar/ModesButton.selected
		3:
			$NaiveOptionsBar.hide()
			$KNNOptionsBar.hide()
			$ClusterOptionsBar.hide()
			$BayesOptionsBar.show()
			mode = $BayesOptionsBar/ModesButton.selected
	pass # Replace with function body.


func _on_classify_button_button_up() -> void:
	if $NaiveOptionsBar.visible == true:
		var naive_node = Naive.new()
		var classe = naive_node.naive_execute(positive_path,negative_path,tweet,k)
		$ResultLabel.text = classe
	elif $KNNOptionsBar.visible == true:
		var knn_node = Knn.new()
		var classe = knn_node.knn_execute(database_path,tweet,k,mode)
		$ResultLabel.text = classe
	elif $ClusterOptionsBar.visible == true:
		var clustering_node = Clustering.new()
		var classe = clustering_node.clustering_execute(database_path,tweet,k,mode)
		$ResultLabel.text = classe
	elif $BayesOptionsBar.visible == true:
		var bayes_node = Bayes.new()
		var classe = bayes_node.bayes_execute(database_path,tweet,mode,representation,ngram_mode)
		$ResultLabel.text = classe
	pass # Replace with function body.


func _on_evaluate_button_button_up() -> void:
	if $NaiveOptionsBar.visible == true:
		var naive_node = Naive.new()
		var tableau_string = naive_node.naive_evaluate(database_path,positive_path,negative_path,k)
		var eval_window = create_evaluation_window()
		eval_window.get_child(0).text = tableau_string
		eval_window.visible = true
	elif $KNNOptionsBar.visible == true:
		var knn_node = Knn.new()
		var tableau_string = knn_node.knn_evaluate(database_path,k,mode)
		var eval_window = create_evaluation_window()
		eval_window.get_child(0).text = tableau_string
		eval_window.visible = true
	elif $ClusterOptionsBar.visible == true:
		var clustering_node = Clustering.new()
		var result = clustering_node.clustering_evaluate(database_path,k,mode)
		print("test")
		var eval_window = create_evaluation_window()
		eval_window.get_child(0).text = result["matrix"]
		eval_window.visible = true
		var image = Image.new()
		var err = image.load_svg_from_string(result["svg"])
		if err == OK:
			var texture = ImageTexture.create_from_image(image)
			var new_window = Window.new()
			new_window.title = "Dendrogram"
			new_window.initial_position = Window.WINDOW_INITIAL_POSITION_CENTER_PRIMARY_SCREEN
			new_window.size = Vector2i(1280,800)
			new_window.visible = false
			new_window.close_requested.connect(_on_close_requested.bind(new_window))
			var new_texture_rect = TextureRect.new()
			new_texture_rect.expand_mode = TextureRect.EXPAND_FIT_WIDTH
			new_texture_rect.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
			new_window.add_child(new_texture_rect)
			add_child(new_window)
			new_texture_rect.texture = texture
			new_window.visible = true
		else:
			print("Erreur lors du chargement du SVG")
	elif $BayesOptionsBar.visible == true:
		var bayes_node = Bayes.new()
		var tableau_string = bayes_node.bayes_evaluate(database_path, mode,representation,ngram_mode)
		var eval_window = create_evaluation_window()
		eval_window.get_child(0).text = tableau_string
		eval_window.visible = true
	pass # Replace with function body.


func _on_positive_button_button_up() -> void:
	$ResultLabel.text = ""
	var filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a positive words file"
	filedialog.file_selected.connect(_on_positive_words_file_selected.bind(filedialog))
	add_child(filedialog)
	filedialog.popup()
	pass # Replace with function body.

func _on_positive_words_file_selected(path,node):
	positive_path = path
	node.queue_free()
	if positive_path.is_empty() or negative_path.is_empty():
		$LaunchButtonsContainer/ClassifyButton.disabled = true
		$LaunchButtonsContainer/EvaluateButton.disabled = true
	else:
		$LaunchButtonsContainer/ClassifyButton.disabled = false
		$LaunchButtonsContainer/EvaluateButton.disabled = false
	pass

func _on_negative_button_button_up() -> void:
	$ResultLabel.text = ""
	var filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a negative words file"
	filedialog.file_selected.connect(_on_negative_words_file_selected.bind(filedialog))
	add_child(filedialog)
	filedialog.popup()
	pass # Replace with function body.

func _on_negative_words_file_selected(path,node):
	negative_path = path
	node.queue_free()
	if positive_path.is_empty() or negative_path.is_empty():
		$LaunchButtonsContainer/ClassifyButton.disabled = true
		$LaunchButtonsContainer/EvaluateButton.disabled = true
	else:
		$LaunchButtonsContainer/ClassifyButton.disabled = false
		$LaunchButtonsContainer/EvaluateButton.disabled = false
	pass

func _on_spin_box_value_changed(value: float) -> void:
	k = int(value)
	$ResultLabel.text = ""
	pass # Replace with function body.


func _on_modes_button_item_selected(index: int) -> void:
	mode = index
	$ResultLabel.text = ""
	pass # Replace with function body.


func _on_representations_button_item_selected(index: int) -> void:
	representation = index
	$ResultLabel.text = ""
	pass # Replace with function body.


func _on_n_gram_modes_button_item_selected(index: int) -> void:
	ngram_mode = index
	$ResultLabel.text = ""
	pass # Replace with function body.
