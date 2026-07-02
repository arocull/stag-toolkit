@tool
extends Node

## Configuration file to load for constructing exports.
@export_global_file("*.json") var export_config: String = "res://export_config.json"
## File to write constructed export presets to.
@export_global_file("*.cfg") var write_to: String = "res://export_presets.cfg"
## Whether to load the existing Export Presets before writing to the configuration file.
@export var use_existing: bool = true
## Offset the preset index, to avoid writing over existing export presets.
@export var start_index: int = 0

var preset_items: Dictionary[String, StagExportPreset] = {}

# TODO: use StagLogger to auto-exit upon error

func _ready() -> void:
	if not Engine.is_editor_hint():
		print("StagToolkit: Exporter must be run in a headless editor to function.")
		return
	if DisplayServer.window_can_draw():
		print("StagToolkit: Editor must be running in headless mode to run exporter.")
		return

	print("StagToolkit: Loading export configuration file...")
	var err: int = OK
	var config = FileAccess.open(export_config, FileAccess.READ)
	err = FileAccess.get_open_error()
	if err != OK or not is_instance_valid(config):
		push_error("Failed to open {0} for reason ({1}): {2}".format([export_config, err, error_string(err)]))
		get_tree().quit(1)

	print("StagToolkit: Parsing config...")
	var config_dict: Dictionary = JSON.parse_string(config.get_as_text())
	print(config_dict)

	# First, build list of exporters
	for key in config_dict.keys():
		print("StagToolkit: Parsing key: ", key)
		build_dependency(config_dict, key)

	var output: ConfigFile = ConfigFile.new()
	if use_existing:
		print("StagToolkit: Getting output file...")
		err = output.load(write_to)
		if err != OK:
			push_error("Failed to load export config")
			get_tree().quit(1)

	# Now setup exports
	var count: int = start_index
	var preset_keys = preset_items.keys()
	print("StagToolkit: Setting up export presets, found these layers: ", preset_keys)
	preset_keys.sort()
	for key in preset_keys:
		var preset := preset_items[key]
		print("StagToolkit: Building preset: ", preset.preset_name)
		for platform in preset.platforms:
			preset.build_preset(output, platform, count)
			count += 1

	print("StagToolkit: Saving export presets...")
	err = output.save(write_to)
	if err != OK:
		push_error("Failed to save configuration")
		get_tree().quit(1)

	#print(output.encode_to_text())

	print("StagToolkit: Done!")
	get_tree().quit(0)


## Builds the given dependency if it doesn't already exist.
func build_dependency(config: Dictionary, key: String) -> StagExportPreset:
	if preset_items.has(key):
		return preset_items[key]

	# Create new dependency
	var dep: StagExportPreset = StagExportPreset.new()
	dep.preset_name = key
	dep.config = config.get(key, {})
	dep.platforms = PackedStringArray(StagUtils.default(dep.config, "platforms", Variant.Type.TYPE_ARRAY, null, true))
	dep.platforms.sort()
	preset_items[key] = dep

	var extends_name: String = StagUtils.default(dep.config, "extends", Variant.Type.TYPE_STRING, "")
	if not extends_name.is_empty():
		dep.extends_preset = build_dependency(config, extends_name)

	return dep

## A configured export preset for multiple platforms.
class StagExportPreset extends RefCounted:
	var preset_name: String
	var extends_preset: StagExportPreset = null
	var config: Dictionary = {}
	var platforms: PackedStringArray = PackedStringArray()

	## Filters a string array, removing duplicates and anything in the given remove list.
	static func filter(data: PackedStringArray, remove: PackedStringArray = PackedStringArray()) -> PackedStringArray:
		var copy = data.duplicate()
		# First, remove all items from the remove list
		for item in remove:
			copy.erase(item)
		# Now, de-duplicate array
		var i: int = 0
		while i < copy.size():
			var ri := copy.rfind(copy[i])
			if ri > i:
				copy.remove_at(ri)
			i += 1
		# Finally, sort array for consistency
		copy.sort()
		return copy

	## Expands a list of filepaths into all files.
	static func expand(pathlist: PackedStringArray) -> PackedStringArray:
		var result: PackedStringArray = PackedStringArray()
		for item in pathlist:
			get_file_list(item, get_file_list(item, result))
		return result

	static func get_file_list(path: String, with: PackedStringArray = []) -> PackedStringArray:
		if FileAccess.file_exists(path):
			with.append(path)
			return with
		with.append_array(StagUtils.walk_directory(path))
		return with

	## Fetches the variable from the configuration.
	func get_config(key: String, data_type: Variant.Type, default: Variant = null) -> Variant:
		if (not config.has(key)) and is_instance_valid(extends_preset):
			return extends_preset.get_config(key, data_type, default)

		var val: Variant = StagUtils.default(config, key, data_type, default, true)

		# Automatically stack string arrays as necessary
		if val is PackedStringArray and is_instance_valid(extends_preset):
			val.append_array(extends_preset.get_config(key, data_type, default))
		return val

	## Returns just a list of
	func get_excluded_files() -> PackedStringArray:
		var files: PackedStringArray = PackedStringArray()
		# Load base file list if necessary
		if extends_preset:
			files = extends_preset.get_excluded_files()
		# Put our excluded files in the exclusion list
		files.append_array(PackedStringArray(get_config("exclude", Variant.Type.TYPE_ARRAY, [])))
		files = StagExportPreset.expand(files)

		# Remove files we forcibly exclude
		var include: PackedStringArray = PackedStringArray(get_config("include", Variant.Type.TYPE_ARRAY, []))
		include = StagExportPreset.expand(include)
		return StagExportPreset.filter(files, include)

	## Builds this preset, appending data to the configuration file.
	func build_preset(cfg: ConfigFile, platform: String, index: int):
		print("Hello world!")
		var preset_name: String = "{0}_{1}".format([preset_name.to_snake_case(), platform.to_snake_case()])
		var export_path: String = get_config("export_path", Variant.Type.TYPE_STRING, "")
		if export_path.is_empty():
			push_warning("Export path for {0} is empty, unable to create directory", preset_name)
		else:
			DirAccess.make_dir_recursive_absolute(export_path.simplify_path())

		var section: String = "preset.{0}".format([index])

		cfg.set_value(section, "name", preset_name)
		cfg.set_value(section, "platform", platform)
		cfg.set_value(section, "advanced_options", true)
		cfg.set_value(section, "export_filter", "exclude")
		cfg.set_value(section, "export_files", get_excluded_files())

		# Encryption
		cfg.set_value(section, "seed", get_config("seed", Variant.Type.TYPE_INT, 0))
		cfg.set_value(section, "encrypt_pck", get_config("encrypt_pck", Variant.Type.TYPE_BOOL, false))
		cfg.set_value(section, "encrypt_directory", get_config("encrypt_directory", Variant.Type.TYPE_BOOL, false))
		cfg.set_value(section, "encryption_include_filters", get_config("encryption_include_filters", Variant.Type.TYPE_STRING, ""))
		cfg.set_value(section, "encryption_exclude_filters", get_config("encryption_exclude_filters", Variant.Type.TYPE_STRING, ""))

		# Feature flags
		cfg.set_value(section, "custom_features", get_config("custom_features", Variant.Type.TYPE_STRING, ""))

		# Basic filters
		cfg.set_value(section, "include_filter", get_config("include_filter", Variant.Type.TYPE_STRING, ""))
		cfg.set_value(section, "exclude_filter", get_config("exclude_filter", Variant.Type.TYPE_STRING, ""))

		# Script export mode
		cfg.set_value(section, "script_export_mode", get_config("script_export_mode", Variant.Type.TYPE_INT, 2))

		cfg.set_value(export_path, "export_path", export_path)


		# NOW, do per-platform options
		var section_options: String = "preset.{0}.options".format([index])
		custom_overrides(cfg, section, section_options)

	func custom_overrides(cfg: ConfigFile, section: String, section_options: String):
		# Do parent overrides first
		if extends_preset:
			extends_preset.custom_overrides(cfg, section, section_options)

		# Automatically pass any unknown keys into the options list
		for key in config.keys():
			match key:
				"extends", "plaftorms", "include", "exclude":
					continue
				_:
					# Make sure section key wasn't already included before
					if not cfg.has_section_key(section, key):
						cfg.set_value(section_options, key, config[key])
