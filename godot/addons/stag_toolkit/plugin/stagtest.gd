extends Node

const DEFAULT_TEST_PATH: String = "res://test/scenarios/"
const DEFAULT_TIMEOUT: float = 30.0

# Singleton for handling tests.

signal test_post_ready()
signal test_pre_exit()

enum ExitCodes {
	Ok = OK,
	Failed = FAILED,
	BadFile = ERR_FILE_UNRECOGNIZED,
}

@onready var args: Dictionary
@onready var statistics: Dictionary = {
	"discovered": 0, # Amount of test files discovered
	"badpaths": 0, # Amount of test files/directories that couldn't be loaded
	"count": 0, # Amount of tests started
	"successes": 0, # Amount of run tests that passed
	"failures": 0, # Amount of run tests that failed
	"skips": 0, # Amount of run tests that skipped
}
@onready var tests: Array[String] = []
@onready var test_idx: int = 0
@onready var active: Node
@onready var active_path: String = ""
@onready var force_exiting: bool = false

func _init():
	process_mode = Node.PROCESS_MODE_ALWAYS
	process_priority = -9223372036854775808
	process_physics_priority = -9223372036854775808

## Engine hook for StagTest.
func _ready():
	# Exit immediately if not a run-time environment.
	if not OS.is_debug_build():
		queue_free()
		return

	args = StagUtils.get_args()
	if args.has("stagtest?"):
		print("StagTest - StagToolkit test harness implementation.")
		print("   flags ---")
		print("\t--stagtest? - displays command output, like this")
		print("\t--stagtest  - runs with StagTest mode")
		print("\t--fast      - escapes on the first test failure, instead of running all tests")
		print("   arguments ---")
		print("\tnote: FILEPATHs can be absolute, relative, or a resource path. Resource paths are strongly advised.")
		print("")
		print("\t--test=FILEPATH - runs the provided scene file, or all files within given directory")
		print("\t\t- if a directory, subdirectories are also run")
		print("\t\t- organized alphabetically within each directory, running subdirectories first")
		print("\t\tFILEPATH=\"{0}\" by default".format([DEFAULT_TEST_PATH]))
		print("\t--timeout=SECONDS - forcibly ends the tests after the given amount of time, returning any collected results")
		print("\t\tSECONDS={0} by default, floating-point times are valid".format([DEFAULT_TIMEOUT]))
		print("")
		__exit()

	# Exit immediately if not a test environment.
	if not args.has("stagtest"):
		queue_free()
		return

	var test_root = args.get("test", DEFAULT_TEST_PATH).replace("\"", "")
	print("StagTest initializing...")

	pause(true)

	# Forcibly exit the given scene
	var scene = get_tree().current_scene
	get_tree().current_scene = null
	if is_instance_valid(scene):
		scene.queue_free()

	# Begin timeout countdown
	var timeout: float = float(args.get("timeout", "{0}".format([DEFAULT_TIMEOUT])))
	get_tree().create_timer(timeout, true, false, true).timeout.connect(
		__force_exit.bind("timeout after {0} seconds".format([timeout])))

	__begin(test_root)

## Begins testing with the given test root.
func __begin(test_root: String):
	print("StagTest - starting with test root \"{0}\"".format([test_root.get_base_dir()]))

	var is_single = FileAccess.file_exists(test_root)
	if is_single:
		tests.append(test_root)
	else:
		__walk_directory(test_root)

	# If we had no tests, go ahead and exit
	if tests.size() == 0:
		__results()
		__exit(ExitCodes.Ok)
		return

	# Otherwise, begin the first test
	test_idx = 0
	__run_test(tests[test_idx])

func __join_path(directory: String, relpath: String) -> String:
	return "{0}/{1}".format([directory, relpath])

## Walks a directory, walking its subdirectories first, then testing every file in the given one.
func __walk_directory(dirpath: String):
	var dir = DirAccess.open(dirpath)
	if !dir:
		print_rich("[color=red]Failed - could not open directory \"{0}\"[/color]".format([dirpath]))
		statistics["badpaths"] += 1
		return

	for subdirpath in dir.get_directories():
		__walk_directory(__join_path(dir.get_current_dir(false), subdirpath))
	for filepath in dir.get_files():
		if filepath.get_extension() == "tscn":
			tests.append(__join_path(dir.get_current_dir(false), filepath).simplify_path())

## Runs a single test at the given filepath.
func __run_test(filepath: String):
	statistics["count"] += 1
	active_path = filepath
	if not ResourceLoader.exists(filepath):
		print_rich("[color=red]Failed - invalid scene file: \"{0}\"[/color]".format([filepath]))
		statistics["badpaths"] += 1
		fail("invalid scene file")
		return

	# TODO: change CACHE_MODE_IGNORE to be CACHE_MODE_IGNORE_DEEP with Godot 4.3+
	# Load scene, ignoring cache when possible to prevent tests stepping on each others' toes
	var packed_scene: PackedScene = ResourceLoader.load(filepath, "PackedScene", ResourceLoader.CACHE_MODE_IGNORE)

	pause(false) # Unpause the tree before test begins
	var status = get_tree().change_scene_to_packed(packed_scene)
	if status != OK:
		fail("failed to initialize scene with error {0}".format([status]))
		return

	active = get_tree().current_scene

func __finish_test():
	pause(true)

	# If we have an ongoing scene but no active test specified, this is probably our active test
	var current = get_tree().current_scene
	if is_instance_valid(current) and not is_instance_valid(active):
		active = current

	# Remove test from the tree
	get_tree().current_scene = null

	# If we have an active test, free it
	if is_instance_valid(active):
		active.queue_free()

	active = null

	if not force_exiting:
		# Begin the next test, if there is one
		test_idx += 1
		if test_idx < tests.size():
			return __run_test(tests[test_idx])

		# Otherwise, finish
		__results()
		__exit(__has_failed())

## Prints the results of a test.
func __results():
	var skipped: bool = statistics.get("skips", 0) > 0

	print("StagTest completed --- {count} tests total".format(statistics))

	var output_good = "\t[color=white]{0}[/color]"
	var output_skip = "\t[color=yellow]{0}[/color]"
	var output_fail = "\t[color=red]{0}[/color]"

	if not skipped:
		output_skip = output_good
	if not __has_failed():
		output_fail = output_good

	print_rich(output_good.format(["{successes} passed"]).format(statistics))
	print_rich(output_skip.format(["{skips} skipped"]).format(statistics))
	print_rich(output_fail.format(["{failures} failed"]).format(statistics))

	if 0 == statistics.get("count", 0): # Warn if no tests were run
		print_rich("\t[color=orange]...in fact, no tests were ran at all!")
	if statistics.get("badpaths", 0) > 0: # Inform of any bad filepaths
		print_rich("\t[color=red]{badpaths} filepaths were considered bad[/color]".format(statistics))

## Exits the runtime.
func __exit(status: int = ExitCodes.Ok):
	pause(true)

	get_tree().quit(status)

## Forcibly exits the runtime, skipping any active tests and returning results.
func __force_exit(reason: String):
	force_exiting = true
	pause(true)
	if is_instance_valid(active):
		skip(reason)

	__results()
	print_rich("[color=orange](this was a forced exit)[/color]")

	# Delay before exiting so objects have time to free themselves
	get_tree().create_timer(0.25, true, false, true).timeout.connect(__exit.bind(ExitCodes.Failed))

## Returns true if any tests have failed.
func __has_failed() -> bool:
	return statistics.get("failures", 0) > 0

### TEST CALLABLES ###

# Immeadiately finishes and passes the given active test.
func finish() -> void:
	statistics["successes"] += 1
	__finish_test()

# Immeadiately skips and ends the test.
func skip(reason: String) -> void:
	statistics["skips"] += 1
	__finish_test()

	print_rich("[color=yellow]SKIPPED \"{0}\" for reason:\n\t{1}[/color]".format([path(), reason]))

# Immeadiately fails and ends the test.
func fail(reason: String) -> void:
	statistics["failures"] += 1
	print_rich("[color=red]FAILED \"{0}\" for reason:\n\t{1}[/color]".format([path(), reason]))

	if args.has("fast"):
		force_exiting = true

	__finish_test()

	if args.has("fast"):
		__force_exit("test failure while in 'fast' mode")

# Performs an assertion.
func assertion(value: bool, message: String = "") -> void:
	# assert(value, message)

	# End test immediately on failure
	if not value:
		fail("assert: " + message)

# Sets the pause of the scene tree
func pause(paused: bool) -> void:
	get_tree().paused = paused

# Returns the path of the active test
func path() -> String:
	return tests[test_idx]
