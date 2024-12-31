extends RefCounted
class_name StagUtils

# Utility functions that are not bundled with Godot, but I wish were.

# Returns a dictionary of command-line arguments used to launch the program.
static func get_args() -> Dictionary:
	var arguments = {}
	for argument in OS.get_cmdline_args():
		if argument.contains("="):
			var key_value = argument.split("=")
			arguments[key_value[0].trim_prefix("--")] = key_value[1]
		else:
			# Options without an argument will be present in the dictionary,
			# with the value set to an empty string.
			arguments[argument.trim_prefix("--")] = ""

	return arguments

# Performs a simple factorial of the given integer.
# Returned as a floating-point for large numbers.
# Returns NAN if n is negative, as it is undefined behavior.
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
