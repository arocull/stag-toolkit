@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
class_name StagUtils
extends RefCounted
## Utility constants and functions that are not bundled with Godot, but I wish were.
## @experimental: Changes may be made for organizational purposes.

## Minimum value for a 64-bit integer.[br]
## Godot uses 64-bit integers by default for [int] types,
## but may use 32-bit in some cases for less memory usage, such as [Vector2i].
const INT64_MIN: int = -9223372036854775808
## Maximum value for a 64-bit integer.[br]
## Godot uses 64-bit integers by default for [int] types,
## but may use 32-bit in some cases for less memory usage, such as [Vector2i].
const INT64_MAX: int = 9223372036854775807
const INT32_MIN: int = -2147483648 ## Minimum value for a 32-bit integer.
const INT32_MAX: int = 2147483647 ## Maximum value for a 32-bit integer.

## Returns a dictionary of command-line arguments used to launch the program.
## Note that all values will be strings.[br][br]
## Example, launching a StagTest scenario:
## [code]godot --headless --stagtest --test=res://test/scenarios/test_island_builder.tscn --timeout=60[/code]
##[codeblock]
##print(StagUtils.get_args())
### outputs
##{
##    "stagtest": "",
##    "test": "res://test/scenarios/test_island_builder.tscn",
##    "timeout": "60"
##}
##[/codeblock]
static func get_args() -> Dictionary[String,String]:
	var arguments: Dictionary[String,String] = {}
	for argument in OS.get_cmdline_args():
		if argument.contains("="):
			var key_value = argument.split("=")
			arguments[key_value[0].trim_prefix("--")] = key_value[1]
		else:
			# Options without an argument will be present in the dictionary,
			# with the value set to an empty string.
			arguments[argument.trim_prefix("--")] = ""
	return arguments

## Performs a simple factorial of the given integer.
## Returned as a floating-point for large numbers.
## Returns [code]NAN[/code] if [code]n[/code] is negative, as it is undefined behavior.
static func factorial(n: int) -> float:
	if n < 0:
		return NAN
	if n == 0 or n == 1:
		return 1

	var sum: float = 1
	while n > 1:
		sum *= n
		n -= 1
	return sum

## Fetches the given key out of the dictionary, or null if not found.[br]
## If the fetched value does not match the specified Variant type ([code]valuetype[/code]),
## the value is forcibly converted to that type (applying default when necessary),
## unless an [code]override[/code] of the same type is specified, at which point the override is used.[br]
## If [code]salvage[/code] is true, similiar types (such as integers and floats)
## are converted instead of using the provided override.
## The override is still applied in cases where types are not similiar (such as string and float).
static func default(
	dictionary: Dictionary, key: Variant, valuetype: Variant.Type,
	override: Variant = null, salvage: bool = true
) -> Variant:
	# Fetch value out of dictionary, or use null if nothing was returned
	var value: Variant = dictionary.get(key, null)

	# If our value was valid, use it!
	if typeof(value) == valuetype:
		return value

	if salvage:
		# Attempt to salvage numbers that are of similiar type
		match valuetype:
			TYPE_INT, TYPE_FLOAT:
				match typeof(value):
					TYPE_INT, TYPE_FLOAT:
						return type_convert(value, valuetype)

	# If our override matches our expected type, use it in place of our invalid value.
	if typeof(override) == valuetype:
		return override

	# Explicit type checks:
	# Using type_convert from a null to a string, returns "<null>"
	# Very nice :)
	match valuetype:
		TYPE_STRING:
			return ""
		TYPE_STRING_NAME:
			return &""
		_: # Otherwise, attempt to convert
			return type_convert(value, valuetype)
