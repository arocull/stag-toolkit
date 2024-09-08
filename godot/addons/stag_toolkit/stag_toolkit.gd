@tool
extends EditorPlugin

enum DockLocation {
	DBottom = 0,
	DDock = 1,
	DContainer = 2,
}

var dockers = [
	#{
		#"scene": preload("res://addons/stag_toolkit/ui/island_docker.tscn"),
		#"title": "Island Builder",
		#"location": DockLocation.DBottom,
	#}
]

var island_builder = preload("res://addons/stag_toolkit/plugin/island_builder.gd").new()

func _enter_tree() -> void:
	for item: Dictionary in dockers:
		if not item.has("scene"):
			push_warning("No configured scene for docker item: ", item)
			continue

		var inst = (item.scene as PackedScene).instantiate()
		item["instance"] = inst


		add_control_to_bottom_panel(inst, item.get("title", "StagToolkit Panel"))

	add_inspector_plugin(island_builder)

func _exit_tree() -> void:
	remove_inspector_plugin(island_builder)

	for item: Dictionary in dockers:
		if item.has("instance"):
			remove_control_from_bottom_panel(item["instance"])
			item["instance"].free()
