extends Control

func _on_log_received(message):
	print(message)
	$RichTextLabel.append_text(message + "\n")
	pass
