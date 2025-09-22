extends Control

func _on_clean_csv_button_button_up():
	$FileDialog.popup()
	pass # Replace with function body.


func _on_file_dialog_file_selected(path):
	print(path)
	$ActionsContainer.hide()
	$CleanLogContainer.show()
	#var new_path = clean_data(path)
	pass # Replace with function body.
