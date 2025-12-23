@tool
@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
extends EditorImportPlugin
## A Gaussian Splat importer. Designed to import .ply files from Scaniverse.
## @experimental

# https://github.com/mkkellogg/GaussianSplats3D/blob/main/src/loaders/ply/INRIAV1PlyParser.js

const SH_C = 0.28209479177387814

func colorize(f: float) -> float:
	return clampf((0.5 + SH_C * f), 0.0, 1.0)
	#return remap(f, -PI, PI, 0.0, 1.0)
func fix_opacity(f: float) -> float:
	return clampf(1.0 - (1.0 / (1.0 + exp(-f))), 0.0, 1.0)
	#return remap(f, -PI, PI, 0.0, 1.0)

enum Presets { DEFAULT }

func _get_importer_name() -> String:
	return "stagtoolkit.gaussian_splat"

func _get_visible_name() -> String:
	return "Gaussian Splat"

func _get_recognized_extensions():
	return ["ply"]

func _get_priority():
	return 1

func _get_import_order():
	return 0

func _get_save_extension():
	return "res"

func _get_resource_type():
	return "MultiMesh"

func _get_preset_count():
	return Presets.size()

func _get_preset_name(preset: int) -> String:
	match preset:
		Presets.DEFAULT:
			return "Default"
		_:
			return "Unknown"

func _get_import_options(_path: String, _preset_index: int) -> Array[Dictionary]:
	return [
		{ ## Runs the IronPress CLI using this configuration to pull and compress textures
			"name": "material_path",
			"default_value": "res://addons/stag_toolkit/utils/shaders/mat_gaussian_splat.tres",
			"property_hint": PROPERTY_HINT_FILE,
			"hint_string": "tres,res"
		},
		{
			"name": "radius",
			"default_value": 2.0,
			"property_hint": PROPERTY_HINT_RANGE,
			"hint_string": "0.0, 10.0, 0.001, or_greater",
		},
	]

func _get_option_visibility(_option: String, _path: StringName, _options: Dictionary) -> bool:
	return true

func _import(
	source_file: String, save_path: String, options: Dictionary,
	_platform_variants: Array[String], gen_files: Array[String]):

	var file: FileAccess = FileAccess.open(source_file, FileAccess.READ)
	if file == null:
		return FileAccess.get_open_error()

	if file.get_line() != "ply" or file.get_error() != OK:
		push_error("Failed to import .ply file, did not start with 'ply' header")
		return ERR_FILE_UNRECOGNIZED

	var vertex_count: int = 0
	var property: PackedStringArray = []

	var err: int = OK
	while file.get_error() == OK:
		var line := file.get_line()
		var line_words := line.split(" ")

		match line_words[0]:
			"property":
				property.append(line_words[2])
			"format":
				match line_words[1]:
					"binary_little_endian":
						file.big_endian = false
					"binary_big_endian":
						file.big_endian = true
					_:
						push_error("Unknown file format: ", line)
						return ERR_FILE_UNRECOGNIZED
			"element":
				if line_words[1] == "vertex":
					vertex_count = int(line_words[2])
			"end_header":
				break # Header ended, we're about to start parsing binary
			_: # Skip
				continue
	if err != OK:
		return err

	if vertex_count <= 0:
		push_error("File does not specify vertex count")
		return ERR_INVALID_PARAMETER

	# Find where all the properties are located
	var index_position: int = -1
	var index_normal: int = -1
	var index_scale: int = -1
	var index_rotation: int = -1
	var index_color: int = -1
	var index_opacity: int = -1
	var property_count: int = property.size()
	for i in range(0, property_count):
		match property[i]:
			"x":
				index_position = i
			"nx":
				index_normal = i
			"f_dc_0":
				index_color = i
			"opacity":
				index_opacity = i
			"scale_0":
				index_scale = i
			"rot_0":
				index_rotation = i

	print("# of Points: ", vertex_count, " | Property Count: ", property_count, " | Properties:\n\t", ",".join(property))

	# Pre-allocate buffers
	var positions: PackedVector3Array = []
	positions.resize(vertex_count)
	var normals: PackedVector3Array = []
	normals.resize(vertex_count)
	var scales: PackedVector3Array = []
	scales.resize(vertex_count)
	var rotations: PackedVector4Array = []
	rotations.resize(vertex_count)
	var colors: PackedVector3Array = []
	colors.resize(vertex_count)
	var opacity: PackedFloat32Array = []
	opacity.resize(vertex_count)

	var i: int = 0
	while err == OK:
		var buffer_vertex: int = i / property_count

		# If we've reached the vertex limit, exit
		if buffer_vertex >= vertex_count:
			#push_warning("{0} ({1}) was greater than {2} * {3}".format([i, idx, vertex_count, property_count]))
			break

		var idx: int = i % property_count
		match idx:
			index_position:
				i += 3
				positions[buffer_vertex] = Vector3(file.get_float(), file.get_float(), file.get_float())
			index_normal:
				i += 3
				normals[buffer_vertex] = Vector3(file.get_float(), file.get_float(), file.get_float())
			index_scale:
				i += 3
				scales[buffer_vertex] = Vector3(exp(file.get_float()), exp(file.get_float()), exp(file.get_float()))
			index_rotation:
				i += 4 # I think the last value is W?
				rotations[buffer_vertex] = Vector4(file.get_float(), file.get_float(), file.get_float(), file.get_float())
			index_color:
				i += 3
				colors[buffer_vertex] = Vector3(file.get_float(), file.get_float(), file.get_float())
			index_opacity:
				i += 1
				opacity[buffer_vertex] = file.get_float()
			_: # Discard float
				i += 1
				file.get_float()
		err = file.get_error()
	if err != ERR_FILE_EOF and err != OK:
		push_error("File read error: ", err)
		return err
	file.close()

	print("Finished parsing splat, generating MultiMesh...")

	# Set up plane mesh for displaying
	var radius: float = options.get("radius", 1.0)
	var plane = PlaneMesh.new()
	plane.orientation = PlaneMesh.FACE_Y
	plane.size = Vector2(radius, radius)

	var material_path: String = options.get("material_path", "")
	if (not material_path.is_empty()) and ResourceLoader.exists(material_path):
		plane.material = ResourceLoader.load(material_path)

	# Begin building multi-mesh
	var mesh: MultiMesh = MultiMesh.new()
	mesh.set_instance_count(0)
	mesh.mesh = plane
	mesh.use_colors = true
	#mesh.use_custom_data = true
	mesh.use_custom_data = false # No data in normals, typically?
	mesh.transform_format = MultiMesh.TRANSFORM_3D
	mesh.physics_interpolation_quality = MultiMesh.INTERP_QUALITY_FAST
	mesh.set_instance_count(vertex_count)

	for j in range(vertex_count):
		var rot := rotations[j]
		var q: Quaternion = Quaternion(rot.x, rot.y, rot.z, rot.w).normalized()
		var t: Transform3D = Transform3D(Basis(q).scaled_local(scales[j]), positions[j])
		mesh.set_instance_transform(j, t)

		# Store normal in custom data
		#var n := normals[j]
		#mesh.set_instance_custom_data(j, Color(n.x, n.y, n.z))

		# Store actual color
		var c := colors[j]
		var color := Color(colorize(c.x), colorize(c.y), colorize(c.z), fix_opacity(opacity[j]))
		# print(color)
		mesh.set_instance_color(j, color)

	var base_path: String = "{0}.{1}".format([save_path, _get_save_extension()])
	return ResourceSaver.save(mesh, base_path)
