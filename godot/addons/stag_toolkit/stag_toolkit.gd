@tool
@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit.svg")
extends EditorPlugin

# StagToolkit © Alan O'Cull 2024
# All-purpose toolkit for Godot game creation.
# See readme.md and LICENSE for details.

# Note: Built-in editor Icon colors can be found here:
# https://github.com/godotengine/godot/blob/4.3/editor/themes/editor_color_map.cpp

## Project Settings config options
var settings: Array[Dictionary] = [
	{
		"name": "addons/stag_toolkit/importers/simple_lod/enabled",
		"type": TYPE_BOOL,
		"description": "Whether the Simple LOD scene importer is enabled or not. Requires plugin reload.",
		"default": true,
	},
	{
		"name": "addons/stag_toolkit/importers/ironpress/enabled",
		"type": TYPE_BOOL,
		"description": "Whether the IronPress material importer is enabled or not. Requires plugin reload.",
		"default": true,
	},
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
	},
	{
		"name": "addons/stag_toolkit/island_builder/collision_color",
		"type": TYPE_COLOR,
		"description": "Debug draw color for generated IslandBuilder collision shapes.",
		"default": Color("#ff00abff"),
	},
	{
		"name": "addons/stag_toolkit/island_builder/save_to_directory",
		"type": TYPE_STRING,
		"hint": PROPERTY_HINT_DIR,
		"description": "Top-level directory to save built Islands in.",
		"default": "art/islands/generated",
	},
	{
		"name": "addons/stag_toolkit/simulated_rope/default_settings",
		"type": TYPE_STRING,
		"hint": PROPERTY_HINT_FILE,
		"description": "Default SimulatedRopeSettings resource when one is not defined by the project developer.",
		"default": "",
	}
]

## Defines where/how the docker is docked
enum DockerType {
	Inspector,
	Import,
	ScenePostImport,
}

## Editor docker configurations
var dockers: Array[Dictionary] = [
	{
		"toggle": "addons/stag_toolkit/island_builder/enabled",
		"resource": "res://addons/stag_toolkit/plugin/island_builder/island_builder.gd",
		"constructed": null,
		"init": "thread_init",
		"deinit": "thread_deinit",
		"type": DockerType.Inspector,
	},
	{
		"toggle": "addons/stag_toolkit/importer/ironpress",
		"resource": "res://addons/stag_toolkit/plugin/importer/ironpress.gd",
		"constructed": null,
		"type": DockerType.Import,
	},
	{
		"toggle": "addons/stag_toolkit/importer/simple_lod",
		"resource": "res://addons/stag_toolkit/plugin/importer/simple_lod.gd",
		"constructed": null,
		"type": DockerType.ScenePostImport,
	}
]

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

	# Register docks and importers
	for docker in dockers:
		if ProjectSettings.get_setting(docker.toggle, true) and docker.has("resource"):
			# Instantiate docker
			var dock = load(docker.resource).new()
			docker["constructed"] = dock

			# Add it as a plugin and initialize
			match docker.type:
				DockerType.Inspector:
					add_inspector_plugin(dock)
				DockerType.Import:
					add_import_plugin(dock)
				DockerType.ScenePostImport:
					add_scene_post_import_plugin(dock)

			if docker.has("init"):
				dock.call(docker.init)

func _exit_tree() -> void:
	# Unregister docks and importers
	for docker in dockers:
		if ProjectSettings.get_setting(docker.toggle, true) and is_instance_valid(docker.get("constructed", null)):
			# Remove plugin and call deconstructor
			match docker.type:
				DockerType.Inspector:
					remove_inspector_plugin(docker.constructed)
				DockerType.Import:
					remove_import_plugin(docker.constructed)
				DockerType.ScenePostImport:
					remove_scene_post_import_plugin(docker.constructed)

			if docker.has("deinit"):
				docker.constructed.call(docker.deinit)

			# Dereference docker for cleanup
			docker.constructed = null
