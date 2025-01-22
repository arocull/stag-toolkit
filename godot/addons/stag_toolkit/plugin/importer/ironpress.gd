@tool
extends EditorImportPlugin

const EXT_MATERIAL = "tres"
## Max time, in milliseconds, to wait for a texture to import before moving on
const TEXTURE_IMPORT_TIMEOUT = 5000

enum Presets { DEFAULT }

func _get_importer_name() -> String:
	return "stagtoolkit.ironpress"

func _get_visible_name() -> String:
	return "IronPress"

func _get_recognized_extensions():
	return ["ironpress"]

func _get_priority():
	return 1

func _get_import_order():
	return 0

func _get_save_extension():
	return "res"

func _get_resource_type():
	return "PlaceholderMaterial"

func _get_preset_count():
	return Presets.size()

func _get_preset_name(preset: int) -> String:
	match preset:
		Presets.DEFAULT:
			return "Default"
		_:
			return "Unknown"

func _get_import_options(_path: String, _preset_index: int):
	return [
	{ ## Runs the IronPress CLI using this configuration to pull and compress textures
		"name": "run_cli",
		"default_value": true,
	},
	{ ## Optimizes textures associated with IronPress file based on the configuration
		"name": "optimize_textures",
		"default_value": true,
	},
	{ ## Creates materials from IronPress file, or overlays existing ones
		"name": "create_materials",
		"default_value": true,
	}
	]

func _get_option_visibility(_option: String, _path: StringName, _options: Dictionary) -> bool:
	return true

func _import(
	source_file: String, save_path: String, options: Dictionary,
	_platform_variants: Array[String], gen_files: Array[String]):

	var file = FileAccess.open(source_file, FileAccess.READ)
	if file == null:
		return FileAccess.get_open_error()

	var content = JSON.parse_string(file.get_as_text())
	if content == null:
		push_error("IronPress Importer: Failed to parse IronPress file: ", source_file)
		return ERR_PARSE_ERROR

	if not content.has("materials"):
		push_error("IronPress Importer: No materials found in IronPress file: ", source_file)
		return ERR_PARSE_ERROR

	# Close file now that we have content
	file.close()
	file = null

	var rfs = EditorInterface.get_resource_filesystem()

	# Run CLI and import associated textures
	var ran_cli: bool = false
	if options.get("run_cli", false):
		var cliout: Array = []
		var error = OS.execute(
			"ironpress",
			[ProjectSettings.globalize_path(source_file)],
			cliout, true, false)
		if error != OK:
			push_warning("IronPress Importer: IronPress CLI failed, skipping step. Output: ", cliout)
		else:
			ran_cli = true

	# Figure out where output textures will be saved
	var textures_dir: String = source_file.get_base_dir().path_join(content["output"]).simplify_path()

	# NOW, ensure our textures are imported
	var reimport_list: PackedStringArray = []
	for mat_name: String in content.get("materials", {}).keys():
		var mat_data: Dictionary = content["materials"][mat_name]
		var channels: Array = mat_data.get("channels", [])

		for channel: String in channels:
			var tex_path: String = textures_dir.path_join("{0}_{1}.png".format([mat_name, channel]))

			# Ensure textures are imported if running CLI
			if ran_cli:
				# Tell editor file system that this file SPECIFICALLY has changed
				rfs.update_file(tex_path)

				# Forcibly import this file RIGHT NOW
				var err = append_import_external_resource(tex_path)
				if err != OK:
					push_error("IronPress Importer: While importing {0}, failed to import texture {1}".format([source_file, tex_path]))
					return err

			# Optimize texture by modifying its settings
			if options.get("optimize_textures", false):
				# Load configuration
				var tex_import_path: String = tex_path + ".import"

				var import_settings: ConfigFile = ConfigFile.new()
				import_settings.load(tex_import_path)

				# Modify import configuration

				# If compression has not been set, enforce it to be 3D with default settings
				# User can override these after
				var is_detect3d: bool = import_settings.get_value("params", "detect_3d/compress_to") == 1
				var is_lossless: bool = import_settings.get_value("params", "compress/mode") == 0
				if is_detect3d and is_lossless:
					import_settings.set_value("params", "compress/mode", 2)
					import_settings.set_value("params", "detect_3d/compress_to", 0)
					import_settings.set_value("params", "mipmaps/generate", true)

				# Forcibly enable/disable normal map
				import_settings.set_value("params", "compress/normal_map", 1 if channel == "normal" else 2)

				# Set source normal if available
				if "normal" in channels and (not channel == "normal"):
					var src_normal: String = textures_dir.path_join("{0}_normal.png".format([mat_name]))
					import_settings.set_value("params", "roughness/src_normal", src_normal)

				# Set roughness mode
				match channel:
					"arm":
						import_settings.set_value("params", "roughness/mode", 3) # Green channel
					"roughness":
						import_settings.set_value("params", "roughness/mode", 2) # Red channel
					_:
						import_settings.set_value("params", "roughness/mode", 1) # Disabled

				# Set color format
				match channel:
					"arm", "ao", "roughness", "metallic", "normal":
						import_settings.set_value("params", "compress/channel_pack", 1)
					_:
						import_settings.set_value("params", "compress/channel_pack", 0)

				import_settings.save(tex_import_path) # Save import settings
				reimport_list.append(tex_path) # We are GOING to reimport this

	# NOW, reimport ALL optimized texture files
	if options.get("optimize_textures", false):
		for item in reimport_list:
			append_import_external_resource(item)

	# FINALLY, import materials
	if options.get("create_materials", false):
		for mat_name: String in content.get("materials", {}).keys():
			var mat_data: Dictionary = content["materials"][mat_name]
			var channels: Array = mat_data.get("channels", [])

			var mat_path: String = "{0}_{1}.{2}".format([source_file.get_basename(), mat_name, EXT_MATERIAL])
			var mat: BaseMaterial3D

			# Load the material if it already exists, and modify it
			# Otherwise, choose whether this is an ARM or PBR material
			# Default to ARM for optimization purposes
			if ResourceLoader.exists(mat_path):
				mat = load(mat_path)
			elif channels.has("arm"):
				mat = ORMMaterial3D.new()
			else:
				mat = StandardMaterial3D.new()

			# Enable transparency if it's used and was not enabled already
			if mat_data.get("alpha", false) and mat.transparency == BaseMaterial3D.TRANSPARENCY_DISABLED:
				mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR

			mat.resource_name = mat_name

			for channel: String in channels:
				# Get texture path
				var orig_path: String = textures_dir.path_join("{0}_{1}.png".format([mat_name, channel]))

				# If don't have the texture, skip over it
				if not ResourceLoader.exists(orig_path):
					push_warning("IronPress Importer: While importing {0}, texture {1} not found".format([source_file, orig_path]))
					continue
				var tex = load(orig_path)

				if is_instance_valid(tex):
					match channel:
						"basecolor":
							mat.albedo_texture = tex
						"arm":
							mat.orm_texture = tex
							mat.ao_enabled = true
						"normal":
							mat.normal_texture = tex
							mat.normal_enabled = true
						"ao":
							mat.ao_texture = tex
							mat.ao_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_RED
							mat.ao_enabled = true
						"roughness":
							mat.roughness_texture = tex
							mat.roughness_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_RED
						"metallic":
							mat.metallic_texture = tex
							mat.metallic_texture_channel = BaseMaterial3D.TEXTURE_CHANNEL_RED
						"emission":
							mat.emission_texture = tex
							mat.emission_enabled = true

			# Save material
			var err = ResourceSaver.save(mat, mat_path)
			if err != OK:
				push_error("IronPress Importer: While importing {0}, failed to save material {1}".format([source_file, mat_name]))
				return err
			gen_files.append(mat_path)

	# Save a placeholder material here, just for Godot to not throw errors
	var base_path: String = "{0}.{1}".format([save_path, _get_save_extension()])
	return ResourceSaver.save(PlaceholderMaterial.new(), base_path)
