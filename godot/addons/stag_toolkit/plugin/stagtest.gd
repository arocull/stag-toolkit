@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
extends Node
## Singleton for handling unit and integration tests.
## @experimental: Fairly solidified, but changes may be made as seen fit.

const DEFAULT_TEST_PATH: String = "res://test/scenarios/"
const DEFAULT_BENCHMARK_PATH: String = "res://test/benchmarks/"
const DEFAULT_REPORTS_PATH: String = "res://test/reports/"
const DEFAULT_TIMEOUT: float = 30.0
const DEFAULT_TIME_SCALE: float = 1.0

## Helper class containing data of a StagTest benchmark results.
class BenchmarkResult extends RefCounted:
	var count: int = 0 ## Number of times benchmark Callable was ran
	var minimum: float = 0 ## Minimum completion time, in milliseconds
	var maximum: float = 0 ## Maximum completion time, in milliseconds
	var mean: float = 0 ## Average completion time, in milliseconds
	var median: float = 0 ## Median completion time, in milliseconds
	var standard_deviation: float = 0 ## Standard deviation of completion time, in milliseconds
	## Converts the benchmark result into a dictionary.
	func dict() -> Dictionary[String,Variant]:
		return {
			"count": self.count,
			"mean": self.mean,
			"median": self.median,
			"standard_deviation": self.standard_deviation,
			"minimum": self.minimum,
			"maximum": self.maximum,
		}
	func _to_string() -> String:
		return "n={0}\tmean={1}\trange=[{2}, {3}]\tmedian={4}\tσ={5}".format([
			self.count,
			StagTest.__format_duration(self.mean),
			StagTest.__format_duration(self.minimum),
			StagTest.__format_duration(self.maximum),
			StagTest.__format_duration(self.median),
			StagTest.__format_duration(self.standard_deviation),
		])

## Testing class for signal expectors.
## This class is thread-safe.
class SignalExpector extends RefCounted:
	var _emitter: Signal
	var _emitter_name: String = ""
	var _count: int = 0
	var _context: String = ""
	var _mutex: Mutex = Mutex.new()

	## Increments the emitter count by 1.
	func _increment(...args: Array[Variant]) -> void:
		_mutex.lock()
		_count += 1
		_mutex.unlock()
	## Asserts that the emitter is not null.
	func assert_valid(extra_context: String = "") -> void:
		StagTest.test_data["assertions"] += 1
		if _emitter.is_null():
			StagTest.fail("expected {0} to not be null{1}{2}".format([
				_emitter_name,
				StagTest.__format_assertion_message(_context),
				StagTest.__format_assertion_message(extra_context)
			]))
	## Asserts that the Signal was emitted at least once.
	func assert_emitted(extra_context: String = "") -> void:
		StagTest.test_data["assertions"] += 1
		if get_count() == 0:
			StagTest.fail("expected {0} to be emitted{1}{2}".format([
				_emitter_name,
				StagTest.__format_assertion_message(_context),
				StagTest.__format_assertion_message(extra_context)
			]))
	## Asserts that the Signal was not emitted at all.
	func assert_not_emitted(extra_context: String = "") -> void:
		StagTest.test_data["assertions"] += 1
		if get_count() > 0:
			StagTest.fail("expected {0} to NOT be emitted{1}{2}".format([
				_emitter_name,
				StagTest.__format_assertion_message(_context),
				StagTest.__format_assertion_message(extra_context)
			]))
	## Asserts that the Signal was emitted exactly `exact_call_count` times.
	func assert_count(exact_call_count: int, extra_context: String = "") -> void:
		StagTest.test_data["assertions"] += 1
		if get_count() != exact_call_count:
			StagTest.fail("expected {0} to be emitted {1} times instead of {2} times{3}{4}".format([
				_emitter_name,
				exact_call_count,
				get_count(),
				StagTest.__format_assertion_message(_context),
				StagTest.__format_assertion_message(extra_context)
			]))
	## Blocks until either the emitter count or timeout (in milliseconds) is reached.
	## Fails the test if the timeout was reached.
	func block_until(threshold: int = 1, timeout_ms: int = 30000, extra_context: String = "") -> void:
		var start := Time.get_ticks_msec()
		while get_count() < threshold:
			if Time.get_ticks_msec() - start >= timeout_ms:
				StagTest.fail(
					"timeout of {0} exceeded while waiting for signal {1} to emit {2} times, but only emitted {3} times{4}{5}".format([
						StagTest.__format_duration(float(timeout_ms) * 1000),
						_emitter_name,
						threshold,
						get_count(),
						_context,
						extra_context,
					])
				)
				return
			OS.delay_usec(1)
	## Resets the expector state.
	func reset() -> void:
		_mutex.lock()
		_count = 0
		_mutex.unlock()
	## Returns the number of times the signal emitted.
	func get_count() -> int:
		return _count

## Testing class for awaiting process/physics_process ticks.
class TickTimer extends RefCounted:
	signal done() ## Emitted after the specified wait ticks have passed.
	var _wait_ticks: int = 0

	## Decreases the wait tick count by 1, and emits upon reaching zero ticks.
	## Returns true if still waiting on ticks.
	static func __decrement(t: TickTimer, wait_until: Signal) -> bool:
		t._wait_ticks -= 1
		if t._wait_ticks == 0:
			wait_until.connect(func (): t.done.emit(), CONNECT_ONE_SHOT)
			return false
		return true
	## Returns the number of wait ticks left.
	func ticks_left() -> int:
		return _wait_ticks
	static func keep(t: TickTimer) -> bool:
		return t.ticks_left() > 0

signal test_post_ready() ## Called just after beginning a test.
signal tick_process() ## Emitted at the beginning of each process tick during the test.
signal tick_physics_process() ## Emitted at the beginning of each physics process tick during the test.
signal internal_tick_process_list_ready()
signal internal_tick_physics_process_list_ready()
signal test_pre_exit() ## Called just before exiting a test.

enum ExitCodes {
	Ok = OK,
	Failed = FAILED,
	BadFile = ERR_FILE_UNRECOGNIZED,
}

var args: Dictionary
var _quit_function: Callable = __quit_default

@onready var statistics: Dictionary = {
	"discovered": 0, # Amount of test files discovered
	"badpaths": 0, # Amount of test files/directories that couldn't be loaded
	"count": 0, # Amount of tests started
	"successes": 0, # Amount of run tests that passed
	"failures": 0, # Amount of run tests that failed
	"skips": 0, # Amount of run tests that skipped
}
@onready var test_data_default: Dictionary = {
	"post_test_message": "",
	"assertions": 0,
}
@onready var test_data: Dictionary = test_data_default.duplicate(true)
@onready var _benchmarks: Dictionary = Dictionary()
@onready var _reports_benchmarks: Dictionary = Dictionary()
@onready var _time_scale_base: float = DEFAULT_TIME_SCALE
@onready var _reports_path: String = DEFAULT_REPORTS_PATH

@onready var tests: Array[String] = []
@onready var test_failures: Array[String] = []
@onready var test_idx: int = 0
@onready var active_path: String = ""
@onready var force_exiting: bool = false
@onready var test_resulted: bool = false
@onready var in_test: bool = false

func _init():
	args = StagUtils.get_args()

	# Always process regardless of engine pause
	process_mode = Node.PROCESS_MODE_ALWAYS
	# This node will ALWAYS process first (int64 minimum)
	process_priority = StagUtils.INT64_MIN
	process_physics_priority = StagUtils.INT64_MIN

## Engine hook for StagTest.
func _ready():
	# Exit immediately if not a run-time environment.
	if not OS.is_debug_build():
		queue_free()
		return

	var default_test_path: String = DEFAULT_TEST_PATH
	if args.has("bench"):
		default_test_path = DEFAULT_BENCHMARK_PATH

	if args.has("stagtest?"):
		print("StagTest - StagToolkit test harness implementation.")
		print("   flags ---")
		print("\t--stagtest? - displays command output, like this")
		print("\t--stagtest  - runs with StagTest mode")
		print("\t--fast      - escapes on the first test failure, instead of running all tests")
		print("\t--bench     - enables benchmark reports and switches default test directory to \"{0}\"".format(
			[DEFAULT_BENCHMARK_PATH]))
		print("\t\t- benchmark times are reported in microseconds, unless otherwise specified")
		print("   arguments ---")
		print("\tnote: FILEPATHs can be absolute, relative, or a resource path. Resource paths are strongly advised.")
		print("")
		print("\t--test=FILEPATH - runs the provided scene file, or all scene files within given directory")
		print("\t\t- if a directory, subdirectories are also run")
		print("\t\t- organized alphabetically within each directory, running subdirectories first")
		print("\t\tFILEPATH=\"{0}\" by default (quotes optional)".format([default_test_path]))
		print("\t--reports=DIRECTORY - writes reports to the given directory")
		print("\t\t- set to empty string \"\" for no reports")
		print("\t\tDIRECTORY=\"{0}\" by default (quotes optional)".format([DEFAULT_REPORTS_PATH]))
		print("\t--timeout=SECONDS - forcibly ends all tests after the given amount of time, returning any collected results")
		print("\t\tSECONDS={0} by default, floating-point times are valid".format([DEFAULT_TIMEOUT]))
		print("\t--timescale=SCALE - sets the default engine time scale when not overidden by tests")
		print("\t\tSCALE={0} by default, floating-point scales are valid".format([DEFAULT_TIME_SCALE]))
		print("")
		__exit()

	# Exit immediately if not a test environment.
	if not args.has("stagtest"):
		queue_free()
		return

	var test_root = args.get("test", default_test_path).replace("\"", "")
	_reports_path = args.get("reports", DEFAULT_REPORTS_PATH).replace("\"", "")
	print("StagTest initializing...")

	# Halt scene processing until tests are ready
	pause(true)

	# Forcibly exit the given scene
	get_tree().unload_current_scene.call_deferred()

	_time_scale_base = float(args.get("timescale", "{0}".format([DEFAULT_TIME_SCALE])))

	# Begin timeout countdown
	var timeout: float = float(args.get("timeout", "{0}".format([DEFAULT_TIMEOUT])))
	get_tree().create_timer(timeout, true, false, true).timeout.connect(__timeout.bind(timeout))

	__begin.call_deferred(test_root)

## Begins testing with the given test root.
func __begin(test_root: String) -> void:
	print("StagTest - Test Root: {0}\n".format([test_root.get_base_dir()]))

	var is_single = FileAccess.file_exists(test_root)
	if is_single:
		tests.append(test_root)
	else:
		__walk_directory(test_root)

	print("")

	# If we had no tests, go ahead and exit
	if tests.size() == 0:
		__results()
		__exit(ExitCodes.Ok)
		return

	# Otherwise, begin the first test
	test_idx = 0
	__run_test(tests[test_idx])

func __join_path(directory: String, relpath: String) -> String:
	return "{0}/{1}".format([directory, relpath]).simplify_path()

## Deferable method for rich printing.
func __print_rich(msg: String) -> void:
	print_rich(msg)

func __display_post_test_message() -> void:
	print_rich(test_data["post_test_message"])

## Walks a directory, walking its subdirectories first, then testing every file in the given one.
func __walk_directory(dirpath: String):
	var dir := DirAccess.open(dirpath)
	if !dir:
		print_rich("[color=red]Failed - could not open directory \"{0}\"[/color]".format([dirpath]))
		statistics["badpaths"] += 1
		return

	for subdirpath in dir.get_directories():
		__walk_directory(__join_path(dir.get_current_dir(false), subdirpath))
	for filepath in dir.get_files():
		if filepath.get_extension() == "tscn" or filepath.get_extension() == "scn":
			tests.append(__join_path(dir.get_current_dir(false), filepath).simplify_path())

## Runs a single test at the given filepath.
func __run_test(filepath: String) -> void:
	statistics["count"] += 1
	active_path = filepath
	print_rich("[color=blue]STARTING TEST {0}[/color]".format([filepath]))
	if not ResourceLoader.exists(filepath):
		print_rich("[color=red]Failed - invalid scene file: \"{0}\"[/color]".format([filepath]))
		statistics["badpaths"] += 1
		fail("invalid scene file")
		return

	# TODO: change CACHE_MODE_IGNORE to be CACHE_MODE_IGNORE_DEEP with Godot 4.3+
	# Load scene, ignoring cache when possible to prevent tests stepping on each others' toes
	var packed_scene: PackedScene = ResourceLoader.load(filepath, "PackedScene", ResourceLoader.CACHE_MODE_IGNORE)

	test_resulted = false
	in_test = true
	test_data = test_data_default.duplicate(true)
	pause(false) # Unpause the tree before test begins
	time_scale(_time_scale_base) # Reset time scale
	var status = get_tree().change_scene_to_packed(packed_scene)
	if status != OK:
		fail("failed to initialize scene with error {0}".format([status]))
		return
	test_post_ready.emit()

## Begins the test cleanup process and ends the test afterward.
func __cleanup_test():
	if not in_test:
		return
	test_pre_exit.emit()
	in_test = false
	pause(true) # Halt all processing

	_tick_timers_process_mu.lock()
	_tick_timers_process.clear()
	_tick_timers_process_mu.unlock()

	_tick_timers_physics_process_mu.lock()
	_tick_timers_physics_process.clear()
	_tick_timers_physics_process_mu.unlock()

	time_scale(_time_scale_base) # Reset time scale
	get_tree().unload_current_scene.call_deferred()

	# After unloading test, pass it if it didn't fail during teardown either
	__pass_test_if_not_failed.call_deferred()

	# Display the post-test message
	__display_post_test_message.call_deferred()

	# Finally, start the next test or finish everything
	__finish_test.call_deferred()

func __finish_test():
	if __has_failed() and args.has("fast"):
		__force_exit("test failure while in 'fast' mode")
		return

	if not force_exiting:
		# Begin the next test, if there is one
		test_idx += 1
		if test_idx < tests.size():
			__run_test.call_deferred(tests[test_idx])
			return

		# Otherwise, teardown
		__results()
		__exit(__has_failed())

## Prints the results of a test.
func __results():
	var skipped: bool = statistics.get("skips", 0) > 0
	var count: int = statistics.get("count", 0)

	statistics["time"] = float(Time.get_ticks_msec()) / 1000

	print("StagTest completed --- {count} tests total in {time} seconds".format(statistics))

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

	# Show all tests that failed
	if test_failures.size() > 0:
		print("\nfailures:")
		for failure in test_failures:
			print_rich("\t{0}".format([failure]))
		print("")

	if 0 == count: # Warn if no tests were run
		print_rich("\t[color=orange]...in fact, no tests were ran at all!")
	if statistics.get("badpaths", 0) > 0: # Inform of any bad filepaths
		print_rich("\t[color=red]{badpaths} filepaths were considered bad[/color]".format(statistics))
	if count > 0 and statistics.get("successes", 0) == count: # Easily highlight that all tests passed
		print_rich("\t[color=green]all tests passed![/color]")

	# Print benchmark results
	if args.has("bench"):
		print("\n\nbenchmarks ---")
		for test_file in _benchmarks.keys():
			print("\t{0}".format([test_file]))
			var benches: Dictionary = _benchmarks.get(test_file, Dictionary())
			for key in benches:
				print("\t\t{0}:\t{1}".format([key, benches.get(key)]))

## Exits the runtime.
func __exit(status: int = ExitCodes.Ok):
	pause(true) # Pause game to prevent further ticks
	__output_reports() # Write reports
	_quit_function.call(status) # Quit

func __timeout(timeout: float):
	__force_exit("timeout after {0} seconds".format([timeout]))

## Forcibly exits the runtime, skipping any active tests and returning results.
func __force_exit(reason: String):
	if force_exiting:
		return
	force_exiting = true
	pause(true)
	if in_test:
		skip(reason)

	__results()
	print_rich("[color=orange](this was a forced exit)[/color]")

	# Delay before exiting so objects have time to free themselves
	get_tree().create_timer(0.25, true, false, true).timeout.connect(__exit.bind(ExitCodes.Failed))

func __pass_test_if_not_failed():
	if not test_resulted:
		statistics["successes"] += 1
		print_rich("[color=green]PASSED {0}[/color] with {1} assertions\n\n".format([path(), test_data["assertions"]]))

## Returns true if any tests have failed.
func __has_failed() -> bool:
	return statistics.get("failures", 0) > 0

## Formats a message with assertion text.
func __format_assertion_message(message: String):
	if message.is_empty():
		return message
	return ": {0}".format([message])

## Formats an value for better readability in assertion strings.
func __format_assertion_value(val: Variant) -> String:
	match typeof(val):
		TYPE_STRING:
			return "\"{0}\"".format([val])
		TYPE_STRING_NAME:
			return "&\"{0}\"".format([val])
		_:
			return str(val)

## Takes a time duration in microseconds, formatting it to a string.
func __format_duration(t: float) -> String:
	if t > 1e6:
		return "%4.4f s" % (t/1e6)
	if t > 1e4:
		return "%4.4f ms" % (t/1e4)
	return "%4.4f μs" % t

## Adds a report for the current test to the reports list.
func __add_report(reports_list: Dictionary, new_report: Variant, label: String):
	var r: Dictionary = reports_list.get(path(), Dictionary()) # Fetch all reports for this test

	# If this report isn't included, add it
	if not reports_list.has(path):
		reports_list[path()] = r

	r[label] = new_report # Set our new report

## Outputs reports to the given directory
func __output_reports():
	# Write no reports if specified not to
	if _reports_path.is_empty():
		return

	# Write benchmark reports
	if args.has("bench"):
		var out: String = __join_path(_reports_path, "benchmarks.json")
		var fail_msg: String = "[color=red]failed to write benchmarks to {0}[/color]".format([out])

		var dirstatus = __ensure_directory(out)
		if not dirstatus == OK:
			print_rich(fail_msg, ": while making directory, error code {0}".format([dirstatus]))

		var benchFile = FileAccess.open(__join_path(_reports_path, "benchmarks.json"), FileAccess.WRITE)
		if not is_instance_valid(benchFile):
			print_rich(fail_msg, ": while opening file, error {0}".format([FileAccess.get_open_error()]))

		print("\nwriting benchmarks to {0}".format([out]))
		benchFile.store_string(JSON.stringify(_reports_benchmarks,"\t",true,true))
		# benchFile.flush()
		benchFile = null # Close benchmark file

func __ensure_directory(filepath: String) -> int:
	var path_absolute = ProjectSettings.globalize_path(filepath.get_base_dir())
	return DirAccess.make_dir_recursive_absolute(path_absolute)

func __quit_default(status: int):
	get_tree().quit(status)

## TICKING ##

var _tick_timers_process: Array[TickTimer] = []
var _tick_timers_physics_process: Array[TickTimer] = []
var _tick_timers_process_mu: Mutex = Mutex.new()
var _tick_timers_physics_process_mu: Mutex = Mutex.new()

func _process(_delta: float) -> void:
	if in_test:
		tick_process.emit()

		# Tick all tick timers down by 1, releasing any that are finished
		_tick_timers_process_mu.lock()
		_tick_timers_process = _tick_timers_process.filter(TickTimer.__decrement.bind(internal_tick_process_list_ready))
		_tick_timers_process_mu.unlock()
		internal_tick_process_list_ready.emit()

func _physics_process(_delta: float) -> void:
	if in_test:
		tick_physics_process.emit()

		# Tick all tick timers down by 1, releasing any that are finished
		_tick_timers_physics_process_mu.lock()
		_tick_timers_physics_process = _tick_timers_physics_process.filter(
			TickTimer.__decrement.bind(internal_tick_physics_process_list_ready))
		_tick_timers_physics_process_mu.unlock()
		internal_tick_physics_process_list_ready.emit()

## SETUP CALLS ##

## Overrides the runtime exit function, in case the game needs additional teardown steps.
func override_exit_function(new_quit: Callable) -> void:
	_quit_function = new_quit

## Returns true if StagTest is testing, in case the game needs to avoid certain setup steps.
func is_active() -> bool:
	return args.has("stagtest")

### TEST CALLABLES ###

## Returns the path of the active test.
func path() -> String:
	return tests[test_idx]

## Sets the pause of the scene tree.
func pause(paused: bool) -> void:
	get_tree().paused = paused

## Sets the engine time scale.
func time_scale(new_scale: float = _time_scale_base) -> void:
	Engine.time_scale = new_scale

## Puts the test into Teardown mode.
## If the test is not skipped or failed during Teardown, it passes.
func teardown() -> void:
	__cleanup_test()

## Puts the test into Teardown mode (if not already), skipping the remainder of the test.
func skip(reason: String) -> void:
	if in_test:
		__cleanup_test()
	if not test_resulted:
		statistics["skips"] += 1
		print_rich("\t[color=yellow]<---- TEST SKIPPED HERE[/color]")
		test_data["post_test_message"] = "[color=yellow]SKIPPED {0} for reason:\n\t{1}[/color]\n\n".format([path(), reason])
		test_resulted = true

## Puts the test into Teardown mode (if not already), marking the test as failed.
func fail(reason: String) -> void:
	if in_test:
		__cleanup_test()
	if not test_resulted:
		statistics["failures"] += 1
		test_failures.append("[color=red]{0}[/color] : {1}".format([path(), reason]))
		print_rich("\t[color=red]<---- TEST FAILED HERE[/color]")

		var backtrace := "[color=orange]"
		for line in Engine.capture_script_backtraces(true):
			backtrace += "\n"+line.format(0, 4)
		backtrace += "[/color]"

		test_data["post_test_message"] = "[color=red]FAILED {0} for reason:[/color]\n\t{1}{2}\n\n".format([
			path(), reason, backtrace
		])
		test_resulted = true

## Assert that a given boolean is true.
func assert_true(value: bool, message: String = "") -> void:
	test_data["assertions"] += 1
	if not value:
		fail("assert wasn't true{0}".format([__format_assertion_message(message)]))

## Assert that two values are equal.
func assert_equal(a: Variant, b: Variant, message: String = "") -> void:
	test_data["assertions"] += 1
	if not a == b:
		fail("assert {0} == {1} wasn't equal{2}".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))

## Assert that two values are NOT equal.
func assert_unequal(a: Variant, b: Variant, message: String = "") -> void:
	test_data["assertions"] += 1
	if a == b:
		fail("assert {0} == {1} was equal{2}".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))

## Assert that the given instance is valid.
func assert_valid(a: Object, message: String = "") -> void:
	test_data["assertions"] += 1
	if not is_instance_valid(a):
		fail("assert {0} was not a valid instance{1}".format([
			__format_assertion_value(a),
			__format_assertion_message(message)]))

## Assert that two values are equal within an epsilon value, that scales with magnitude.[br]
## [b]Note[/b]: to use a specific delta threshold value, use [code]StagTest.assert_in_delta(...)[/code] instead.
func assert_approx_equal(a: Variant, b: Variant, message: String = "") -> void:
	test_data["assertions"] += 1

	# Ensure types match
	if typeof(a) != typeof(b):
		fail("assert {0} ~= {1} had mismatch types".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))
		return

	var approx_equal: bool = false
	if a is float or a is int:
		approx_equal = is_equal_approx(a, b)
	elif (a is Vector2 or a is Vector2i or a is Vector3 or a is Vector3i or a is Vector3 or a is Vector4i or
		a is Quaternion or a is Basis or a is Transform2D or a is Transform3D or a is Plane or a is Color):
		approx_equal = a.is_equal_approx(b)
	else:
		fail("assert {0} ~= {1} were not supported type".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))
		return

	if not approx_equal:
		fail("assert {0} ~= {1} were not approximately equal".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))

## Assert that two values are equal, within a threshold amount.
## Use [code]StagTest.assert_approx_equal()[/code] if the delta must scale with magnitude.[br][br]
## For floating-point vectors, the overall distance between vectors is compared.
## For integer vectors, Manhattan distance is used instead.
func assert_in_delta(a: Variant, b: Variant, delta: float = 1e-5, message: String = "") -> void:
	test_data["assertions"] += 1

	# Ensure types match
	if typeof(a) != typeof(b):
		fail("assert Δ >= | {0} - {1} | had mismatch types".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			__format_assertion_message(message)]))

	var diff: float = INF
	var approximately_equal: bool = is_same(a, b)
	if not approximately_equal:
		if a is float or a is int:
			diff = abs(a - b)
		elif a is Vector2 or a is Vector3 or a is Vector4: # Regular distance check
			diff = a.distance_to(b)
		elif a is Vector2i: # Otherwise, use Manhattan distance
			diff = absi(a.x - b.x) + absi(a.y - b.y)
		elif a is Vector3i:
			diff = absi(a.x - b.x) + absi(a.y - b.y) + absi(a.z - b.z)
		elif a is Vector4i:
			diff = absi(a.x - b.x) + absi(a.y - b.y) + absi(a.z - b.z) + absi(a.w - b.w)
		else:
			fail("assert Δ >= | {0} - {1} | were not a supported type {4}".format([
				__format_assertion_value(a),
				__format_assertion_value(b),
				delta, diff, __format_assertion_message(message)]))

		approximately_equal = diff <= delta

	# Return if failed
	if not approximately_equal:
		fail("assert Δ >= | {0} - {1} | were not in delta ({2} < {3}) {4}".format([
			__format_assertion_value(a),
			__format_assertion_value(b),
			delta, diff, __format_assertion_message(message)]))


## Creates a [StagTest.SignalExpector] from the given signal, which can be used for further assertions.
## The [StagTest.SignalExpector] is thread-safe.
## Fails the test if the signal is null, or if the signal could not be connected.
func signal_expector(emitter: Signal, message: String = "") -> SignalExpector:
	var expector := SignalExpector.new()

	if emitter.is_null():
		StagTest.fail("while creating Signal Expector, expected emitter to exist{0}".format([
			StagTest.__format_assertion_message(message)
		]))
		return expector

	expector._emitter = emitter
	expector._emitter_name = emitter.get_name()
	expector._context = message

	var err: int
	err = emitter.connect(expector._increment)

	if err != OK:
		StagTest.fail("while creating Signal Expector, failed to connect to emitter with error:{0}\n{1}".format([
			error_string(err),
			StagTest.__format_assertion_message(message)
		]))
		return expector

	return expector

## Returns a signal that emits after the given amount of process ticks.
## If [code]ticks[/code] is less than or equal to zero, the signal emits on the next process tick.
##
##[codeblock]
##await StagTest.tick_timer_process(10) # Wait 10 ticks
### Resumes coroutine at the very beginning of tick 10 before anything else processes
##StagTest.assert_equal(10, process_ticks_stagtest, "awaited 10 process ticks")
##StagTest.assert_equal(9, process_ticks_node, "nodes are just about to process tick 10")
##[/codeblock]
func tick_timer_process(ticks: int) -> Signal:
	if ticks <= 1:
		return tick_process

	var timer := TickTimer.new()
	timer._wait_ticks = ticks
	_tick_timers_process_mu.lock()
	_tick_timers_process.append(timer)
	_tick_timers_process_mu.unlock()
	return timer.done

## Returns a signal that emits after the given amount of physics process ticks.
## If [code]ticks[/code] is less than or equal to zero, the signal emits on the next physics tick.
##
##[codeblock]
##await StagTest.tick_timer_physics_process(10) # Wait 10 ticks
### Resumes coroutine at the very beginning of tick 10 before anything else processes
##StagTest.assert_equal(10, physics_ticks_stagtest, "awaited 10 process ticks")
##StagTest.assert_equal(9, physics_ticks_node, "nodes are just about to process tick 10")
##[/codeblock]
func tick_timer_physics_process(ticks: int) -> Signal:
	if ticks <= 1:
		return tick_physics_process

	var timer := TickTimer.new()
	timer._wait_ticks = ticks
	_tick_timers_physics_process_mu.lock()
	_tick_timers_physics_process.append(timer)
	_tick_timers_physics_process_mu.unlock()
	return timer.done

## Performs a timing benchmark of the Callable (with no arguments) the specified number of times, returning an analysis.[br]
## If timeout is greater than zero, forcibly stops benchmark after X many seconds.
## If a test is skipped or failed during the benchmark, the benchmark exits without completing all iterations.[br]
## Results are always in microseconds, unless otherwise specified.[br]
## Use the [code]--bench[/code] flag when running to output benchmark results.[br]
## [br]
## [b]Note[/b]: Requires the compiled Rust backend.
## @experimental
func benchmark(f: Callable, count: int, label: String, timeout: float = -1) -> BenchmarkResult:
	var failure: bool = false
	if not ClassDB.class_exists("QueueFloat"):
		fail("for benchmark \"{0}\": Rust backend must be included for QueueFloat".format([label]))
		failure = true
	if count <= 0:
		fail("for benchmark \"{0}\": benchmark count must be greater than 0".format([label]))
		failure = true

	if failure: # Return empty report on failure
		var res = BenchmarkResult.new()
		__add_report(_benchmarks, res, label)
		__add_report(_reports_benchmarks, res.dict(), label)
		return res

	# Initialize float queue and store timings
	var queue = ClassDB.instantiate("QueueFloat")
	queue.allocate(count)

	var iterations: int = 0 # Number of times we've ran the Callable
	var goal_time: int = -1
	if timeout > 0:
		goal_time = Time.get_ticks_msec() + int(timeout * 1000)

	for i in range(0, count):
		iterations += 1
		var start: int = Time.get_ticks_usec()
		f.call()
		queue.push(float(Time.get_ticks_usec() - start))

		# Stop benchmarking if the test has ended
		if not in_test:
			break
		# Stop benchmarking if we exceeded our timeout
		if goal_time > 0 and Time.get_ticks_msec() > goal_time:
			break

	# Perform analysis and return results
	var res = BenchmarkResult.new()
	res.count = iterations
	res.mean = queue.mean()
	res.median = queue.median()
	var timerange: Vector2 = queue.range()
	res.minimum = timerange.x
	res.maximum = timerange.y
	res.standard_deviation = queue.standard_deviation()

	__add_report(_benchmarks, res, label)
	__add_report(_reports_benchmarks, res.dict(), label)

	return res
