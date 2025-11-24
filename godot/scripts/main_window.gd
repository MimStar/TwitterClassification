extends Control

const actions_scene = preload("res://scenes/actions_container.tscn")
const clean_log_scene = preload("res://scenes/clean_log_container.tscn")
const knn_scene = preload("res://scenes/knn_container.tscn")
const cluster_scene = preload("res://scenes/cluster_container.tscn")
const bayes_scene = preload("res://scenes/bayes_container.tscn")

var actions_container
var logs_container
var knn_container
var cluster_container
var bayes_container
var clean_data
var filedialog

var positive_path
var negative_path

func _ready():
	add_actions_container()
	pass

func add_actions_container():
	actions_container = actions_scene.instantiate()
	add_child(actions_container)
	actions_container.get_node("FlowContainer/CleanCSVButton").button_up.connect(_on_clean_csv_button_button_up)
	actions_container.get_node("FlowContainer/AnnotateButton").button_up.connect(_on_annotate_button_button_up)
	actions_container.get_node("FlowContainer/KNNButton").button_up.connect(_on_knn_button_button_up)
	actions_container.get_node("FlowContainer/ClusteringButton").button_up.connect(_on_cluster_button_button_up)
	actions_container.get_node("FlowContainer/BayesButton").button_up.connect(_on_bayes_button_button_up)
	pass

func add_logs_container():
	actions_container.queue_free()
	logs_container = clean_log_scene.instantiate()
	add_child(logs_container)
	pass

func add_knn_container():
	actions_container.queue_free()
	knn_container = knn_scene.instantiate()
	add_child(knn_container)

func add_cluster_container():
	actions_container.queue_free()
	cluster_container = cluster_scene.instantiate()
	add_child(cluster_container)

func add_bayes_container():
	actions_container.queue_free()
	bayes_container = bayes_scene.instantiate()
	add_child(bayes_container)

func _on_clean_csv_button_button_up():
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a csv file that contains tweets"
	filedialog.file_selected.connect(_on_clean_csv_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass # Replace with function body.
	
func _on_annotate_button_button_up():
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a positive words file"
	filedialog.file_selected.connect(_on_positive_words_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass
	
func _on_knn_button_button_up():
	add_knn_container()
	pass

func _on_cluster_button_button_up():
	add_cluster_container()
	pass
	
func _on_bayes_button_button_up():
	add_bayes_container()
	pass

func _on_clean_csv_file_selected(path):
	add_logs_container()
	clean_data = CleanData.new()
	clean_data.log_sent.connect(logs_container._on_log_received)
	var new_path = clean_data.clean_data(path)
	print(new_path)
	pass # Replace with function body.
	
func _on_positive_words_file_selected(path):
	positive_path = path
	filedialog.queue_free()
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a negative words file"
	filedialog.file_selected.connect(_on_negative_words_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass
	
func _on_negative_words_file_selected(path):
	negative_path = path
	filedialog.queue_free()
	filedialog = FileDialog.new()
	filedialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	filedialog.access = FileDialog.ACCESS_FILESYSTEM
	filedialog.title = "Open a csv file to annotate"
	filedialog.file_selected.connect(_on_annotate_file_selected)
	add_child(filedialog)
	filedialog.popup()
	pass
	
func _on_annotate_file_selected(path):
	#TODO: Lancer l'annotation
	pass
