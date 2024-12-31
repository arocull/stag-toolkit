@tool
extends EditorPlugin

# StagToolkit © Alan O'Cull 2024
# All-purpose toolkit for Godot game creation.
# See readme.md and LICENSE for details.

# List of StagToolkit classes for registration
# Note: Built-in editor Icon colors can be found here:
# https://github.com/godotengine/godot/blob/4.3/editor/themes/editor_color_map.cpp
var classes: Array[Dictionary] = [
	{
		"type": "StagUtils",
		"base": "RefCounted",
		"script": "res://addons/stag_toolkit/plugin/utils.gd",
		"icon": "res://addons/stag_toolkit/icons/icon_islandbuilder.svg",
		"debug": false,
	},
	{
		"type": "StagImportUtils",
		"base": "RefCounted",
		"script": "res://addons/stag_toolkit/plugin/utils_import.gd",
		"icon": "res://addons/stag_toolkit/icons/icon_islandbuilder.svg",
		"debug": true,
	}
]

# List of Dockers
## IslandBuilder
var island_builder = preload("res://addons/stag_toolkit/plugin/island_builder.gd").new()

# List of Importers
## Simple LOD
var import_simple_lod = preload("res://addons/stag_toolkit/plugin/importer/simple_lod.gd").new()

func _enter_tree() -> void:
	# Register all custom classes
	var class_order = classes.duplicate(false) # Duplicate array to preserve order
	for custom in class_order:
		# Skip item if it is debug only
		if custom.get("debug", false) and not OS.is_debug_build():
			continue
		add_custom_type(custom.type, custom.base, load(custom.script), load(custom.icon))
		print("Registered ", custom.type)

	# Register docks

	## Island Builder
	add_inspector_plugin(island_builder)
	island_builder.thread_init()

	# Register importers

	## Simple LOD
	add_scene_post_import_plugin(import_simple_lod)

func _exit_tree() -> void:
	# Unregister importers

	## Simple LOD
	remove_scene_post_import_plugin(import_simple_lod)

	# Unregister docks

	## Island Builder
	island_builder.thread_deinit()
	remove_inspector_plugin(island_builder)

	# Unregister all custom classes
	var class_order = classes.duplicate(false) # Duplicate array to preserve order
	class_order.reverse()
	for custom in class_order:
		# Skip item if it is debug only
		if custom.get("debug", false) and not OS.is_debug_build():
			continue
		remove_custom_type(custom.type)
