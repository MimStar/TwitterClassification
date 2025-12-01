extends Control

const main_theme = preload("res://assets/main_theme.tres")

var csv_path = ""
var data = []

signal csv_success
signal csv_error

func _ready():
	#Récupérer les données du CSV clean
	load_csv(csv_path)
	#Ajout du Scroll Container
	var scroll_container = ScrollContainer.new()
	scroll_container.set_anchors_and_offsets_preset(Control.PRESET_FULL_RECT)
	$table.add_child(scroll_container)
	#Ajout du Grid Container
	var grid_container = GridContainer.new()
	grid_container.theme = main_theme
	grid_container.columns = data[0].size()
	print(grid_container.columns)
	scroll_container.add_child(grid_container)
	grid_container.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	grid_container.size_flags_vertical = Control.SIZE_EXPAND_FILL
	#Ajout des cellules
	for row_index in range(data.size()):
		for col_index in range(data[row_index].size()):
			var line_edit = LineEdit.new()
			line_edit.text = data[row_index][col_index]
			line_edit.theme = main_theme
			line_edit.expand_to_text_length = true
			line_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			line_edit.size_flags_vertical = Control.SIZE_EXPAND_FILL
			line_edit.set_meta("row", row_index)
			line_edit.set_meta("column", col_index)
			line_edit.text_changed.connect(_on_line_edit_text_changed.bind(line_edit))
			grid_container.add_child(line_edit)
	pass
	
func load_csv(file_path: String) -> void:
	var file = FileAccess.open(file_path, FileAccess.READ)
	data = []
	if file:
		while not file.eof_reached():
			var line = file.get_csv_line()
			if line.size() > 0 and line[0] != "":
				data.append(line)
		file.close()
	
func _on_line_edit_text_changed(new_text: String, line_edit : LineEdit):
	data[line_edit.get_meta("row")][line_edit.get_meta("column")] = new_text
	pass

func _on_save_button_button_up() -> void:
	if csv_path == "":
		csv_error.emit()
		return
	
	save_csv(csv_path)

func save_csv(file_path: String) -> void:
	var file = FileAccess.open(file_path, FileAccess.WRITE)
	
	if file:
		for row in data:
			# Convert the row array to a CSV line
			var csv_line = []
			for cell in row:
				# Escape the cell if it contains commas, quotes, or newlines
				var escaped_cell = str(cell)
				if escaped_cell.find(",") != -1 or escaped_cell.find("\"") != -1 or escaped_cell.find("\n") != -1:
					escaped_cell = "\"" + escaped_cell.replace("\"", "\"\"") + "\""
				csv_line.append(escaped_cell)
			
			# Write the CSV line
			file.store_csv_line(csv_line)
		
		file.close()
		csv_success.emit()
	else:
		csv_error.emit()
