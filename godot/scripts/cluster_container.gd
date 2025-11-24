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
	else:
		$LaunchButton.disabled = false
	pass

func _on_launch_button_button_up() -> void:
	var clustering_node = Clustering.new()
	var visualization = clustering_node.hierarchical_execute(database_path,k,mode)
	var image = Image.new()
	var err = image.load_svg_from_string(visualization)
	if err == OK:
		var texture = ImageTexture.create_from_image(image)
		$DendrogramWindow/TextureRect.texture = texture
		$DendrogramWindow.visible = true
	else:
		print("Erreur lors du chargement du SVG")
	pass # Replace with function body.

func _on_dendrogram_window_close_requested() -> void:
	$DendrogramWindow.visible = false
	pass # Replace with function body.
