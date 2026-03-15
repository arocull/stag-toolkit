@tool
extends ShaderMaterial
class_name PreprocessorShaderMaterial
## A [ShaderMaterial] that generates its [Shader] using a list of defines and includes.
## Useful for shaders that you want to be highly-configurable, but optimized, by using preprocessors.
##
## The [PreprocessorShaderMaterial] is primarily intended for automatic handling of
## preprocessor defines within a [ShaderMaterial].
## The bulk of your shader code is expected to be written in [ShaderInclude]s.
## [br][br]
## A new [Shader] resource is created when any preprocessors (flags, constants, includes) are modified in the inspector,
## in order to force Godot to recompile the shader.
## You can also manually rebuild the shader with the [method PreprocessorShaderMaterial.build_shader] method..
## [br][br]
## Generated shaders are laid out in this order:
## [b]Shader Type[/b],
## [b]Defined Constants[/b] (sorted alphabetically),
## [b]Defined Flags[/b] (sorted alphabetically),
## [b]Prelude[/b],
## [b]Includes[/b] (using the provided order).

## Emitted when the shader is initially built or rebuilt.
signal rebuilt()

# Variable name prefix for toggleable defines
const _FLAG_PREFIX = "flag_"
# Generated shader structure
const _STRUCTURE = "shader_type {shader_type};\n// Constants\n{constants}\n// Flags\n{defines}\n// Prelude\n{prelude}\n//Includes\n{includes}\n"

# Force the shader to rebuild in case it is not automatically updating.
@export_tool_button("Force Rebuild") var _build_shader = build_shader

@export_group("Shader Setup")
## The shader type to generate.
@export var shader_type: Shader.Mode = Shader.Mode.MODE_SPATIAL:
	set(newval):
		shader_type = newval
		build_shader()

## Any code you want to include in the shader before the list of includes.
## Placed after the defines.
@export_multiline var prelude: String = "":
	set(newval):
		prelude = newval
		build_shader()

## An ordered list of shader includes to include in the shader, after defines.
@export var includes: Array[ShaderInclude] = []:
	set(newval):
		includes = newval
		_update_flags_available()
		build_shader()

## A list of shader defines that can be toggled under the Flags group.
@export_storage var flags_available: PackedStringArray = PackedStringArray():
	set(newval):
		flags_available = newval
		notify_property_list_changed()

@export_group("Constants")
## Constant values to be used as defines in the shader.
## [br]
## Currently only [int], [float], [bool], and [String] types are supported.
@export var constants: Dictionary[String, Variant] = {}:
	set(newval):
		# Debounce so typing into the inspector does not trigger regens
		if newval != constants:
			constants = newval
			build_shader()

@export_group("Flags", _FLAG_PREFIX)
## A list of all defines that should be enabled in the shader.
## [br]
## If changed from code, remember to call [method PreprocessorShaderMaterial.build_shader] to rebuild the shader.
@export_storage var flags_enabled: PackedStringArray = PackedStringArray()

# Automatically build the list of defines available from the list of shader includes.
# This does not recursive through shader includes not provided in the list.
@export_tool_button("Fetch Available Flags") var _fetch_defines = _update_flags_available
func _update_flags_available() -> void:
	flags_available = fetch_available_defines()

## Sets the given flag as enabled or disabled.
## Returns [code]true[/code] if the preprocessor list changed.
## [br]
## Call [method PreprocessorShaderMaterial.build_shader] afterward to force a recompile.
func set_flag(flag: String, enabled: bool) -> bool:
	if enabled and flag not in flags_enabled:
		flags_enabled.append(flag)
		flags_enabled.sort()
		return true
	if (not enabled) and flag in flags_enabled:
		flags_enabled.erase(flag)
		return true
	return false

## Sets the named constant to the given value.
## [br]
## Call [method PreprocessorShaderMaterial.build_shader] afterward to force a recompile.
func set_constant(constant: String, value: Variant):
	constants[constant] = value
## Removes the named constant. Returns [code]true[/code] if the given constant existed in the dictionary.
## [br]
## Call [method PreprocessorShaderMaterial.build_shader] afterward to force a recompile.
func remove_constant(constant: String) -> bool:
	return constants.erase(constant)

# https://docs.godotengine.org/en/stable/classes/class_object.html#class-object-private-method-get-property-list
func _get_property_list() -> Array[Dictionary]:
	var properties: Array[Dictionary] = []

	for flag_name in flags_available:
		properties.append({
			"name": "{0}{1}".format([_FLAG_PREFIX, flag_name]),
			"type": TYPE_BOOL,
			#"hint": PROPERTY_HINT_BOO,
		})

	return properties

func _get(property: StringName):
	if property.begins_with(_FLAG_PREFIX):
		return property.substr(_FLAG_PREFIX.length()) in flags_enabled

func _set(property: StringName, value):
	print(property, "\t", value)
	if property.begins_with(_FLAG_PREFIX):
		var flag_name := property.substr(_FLAG_PREFIX.length())

		if set_flag(flag_name, value):
			notify_property_list_changed()
			build_shader()

		return true
	return false

## Returns a string for the given shader type.
static func shader_type_string(st: Shader.Mode) -> String:
	match st:
		Shader.Mode.MODE_CANVAS_ITEM:
			return "canvas_item"
		Shader.Mode.MODE_PARTICLES:
			return "particles"
		Shader.Mode.MODE_SKY:
			return "sky"
		Shader.Mode.MODE_FOG:
			return "fog"
		Shader.Mode.MODE_SPATIAL, _:
			return "spatial"

func _init():
	build_shader.call_deferred()

## Rebuilds the shader from scratch.
## In order for Godot to recompile the shader code, a new [Shader] is created.
## [br]
## Frequently rebuilding the shader can have a negative impact on performance, so only call this when necessary.
func build_shader():
	var newShader: Shader = Shader.new()

	var constants_list: PackedStringArray = PackedStringArray()
	for key in constants.keys():
		var val = constants[key]
		if val is float or val is int or val is bool or val is String:
			constants_list.append("#define {0} {1}".format([key, str(val)]))
		else:
			push_warning("Unsupported shader constant: {0} is type {1}", key, type_string(typeof(val)))
	constants_list.sort() # Sort for consistency

	var flags_list: PackedStringArray = PackedStringArray()
	for defname in flags_enabled:
		flags_list.append("#define {0}".format([defname]))
	flags_list.sort() # Sort for consistency

	var includes_list: PackedStringArray = PackedStringArray()
	for include in includes:
		includes_list.append("#include \"{0}\"".format([include.resource_path]))
	# Do not sort includes list, order is important

	# Finally construct shader
	newShader.set_code(_STRUCTURE.format({
		"shader_type": PreprocessorShaderMaterial.shader_type_string(shader_type),
		"constants": "\n".join(constants_list),
		"defines": "\n".join(flags_list),
		"prelude": prelude.strip_edges(true, true),
		"includes": "\n".join(includes_list)
	}))

	# Swap shader (Godot should recompile it)
	shader = newShader

	rebuilt.emit()

## Searches through all provided shader includes, and returns a list of toggleable shader defines.
func fetch_available_defines() -> PackedStringArray:
	var defines: PackedStringArray = PackedStringArray()

	# Build a regex to search for "ifdef" cases
	var search = RegEx.new()
	var err := search.compile("(#ifdef|#ifndef|#undef)\\s+([a-zA-Z0-9_]+)\\s+?")
	if err != OK:
		push_error("Failed to compile Regex: ", error_string(err))
		return flags_available

	# Iterate over all includes and find potential defines
	for include in includes:
		if include.code.length() == 0:
			push_warning("Shader Include {0} is empty".format([include.resource_path]))
		var matches := search.search_all(include.code)
		for potential_define in matches:
			# Make sure our capture group is filled
			if potential_define.strings.size() >= 3:
				# Fetch the define name, and strip any whitespace characters we might have
				var defname := potential_define.strings[2].strip_edges(true, true)
				# If we didn't already include the define, grab it
				if defname not in defines:
					defines.append(defname)

	return defines
