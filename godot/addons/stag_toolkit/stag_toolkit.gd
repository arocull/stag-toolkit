@tool
extends EditorPlugin

# StagToolkit © Alan O'Cull 2024
# All-purpose toolkit for Godot game creation.
# See readme.md and LICENSE for details.

# Note: Built-in editor Icon colors can be found here:
# https://github.com/godotengine/godot/blob/4.3/editor/themes/editor_color_map.cpp

## Project Settings config options
var settings: Array[Dictionary] = [
	{
		"name": "addons/stag_toolkit/island_builder/enabled",
		"type": TYPE_BOOL,
		"description": "Whether the IslandBuilder tool is enabled or not. Requires plugin reload.",
		"default": true,
	},
	{
		"name": "addons/stag_toolkit/island_builder/render_layers",
		"type": TYPE_INT,
		"hint": PROPERTY_HINT_LAYERS_3D_RENDER,
		"description": "What render layers that newly generated IslandBuilder meshes will appear on.",
		"default": 5,
	}
]

## Defines where the docker is docked
enum DockerType {
	Inspector = 0,
}

## Editor docker configurations
var dockers: Array[Dictionary] = [
	{
		"toggle": "addons/stag_toolkit/island_builder/enabled",
		"resource": "res://addons/stag_toolkit/plugin/island_builder.gd",
		"constructed": null,
		"init": "thread_init",
		"deinit": "thread_deinit",
		"type": DockerType.Inspector,
	}
]

# List of Importers
## Simple LOD
var import_simple_lod = preload("res://addons/stag_toolkit/plugin/importer/simple_lod.gd").new()

## Initializes all configuration options for StagToolkit
func initialize_settings() -> void:
	for setting in settings:
		# Create the setting if it does not already exist
		if not ProjectSettings.has_setting(setting.name):
			ProjectSettings.set_setting(setting.name, setting.default)
		# Add property info for the setting
		ProjectSettings.add_property_info(setting)

func _enter_tree() -> void:
	for setting in settings:
		if not ProjectSettings.has_setting(setting.name):
			ProjectSettings.set_setting(setting.name, setting.default)
		ProjectSettings.add_property_info(setting)

	# Register docks
	for docker in dockers:
		if ProjectSettings.get_setting(docker.toggle, true) and docker.has("resource"):
			# Instantiate docker
			var dock = load(docker.resource).new()
			docker["constructed"] = dock

			# Add it as a plugin and initialize
			add_inspector_plugin(dock)
			if docker.has("init"):
				dock.call(docker.init)

	# Register importers

	## Simple LOD
	add_scene_post_import_plugin(import_simple_lod)

func _exit_tree() -> void:
	# Unregister importers

	## Simple LOD
	remove_scene_post_import_plugin(import_simple_lod)

	# Unregister docks
	for docker in dockers:
		if ProjectSettings.get_setting(docker.toggle, true) and is_instance_valid(docker.get("constructed", null)):
			# Remove plugin and call deconstructor
			remove_inspector_plugin(docker.constructed)
			if docker.has("deinit"):
				docker.constructed.call(docker.deinit)

			# Dereference docker for cleanup
			docker.constructed = null
