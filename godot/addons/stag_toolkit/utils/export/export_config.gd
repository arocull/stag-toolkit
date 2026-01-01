@tool
extends Node

## Configuration file to load for constructing exports.
@export_global_file("*.json") var export_config: String = "export_config.json"
## File to write constructed export presets to.
@export_global_file("*.cfg") var write_to: String = "res://export_presets.cfg"

func _ready() -> void:

	var output: ConfigFile = ConfigFile.new()


	pass
