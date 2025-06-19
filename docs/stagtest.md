# StagTest

Integration test harness for Godot Engine, provided by StagToolkit.

- [Getting Started](#getting-started)
- [Methods](#methods)
- [FAQ](#faq)

## Getting Started

It's heavily recommended to add your version-of-choice Godot Engine binary to your PATH, so it can be run like so: `$ godot`

To use StagTest, add the script `res://addons/stag_toolkit/plugin/stagtest.gd` as `StagTest` to your project's list of Auto-Loads.
The order should not matter, but I recommend loading it last.

To verify that StagTest is working, run it while inside your Godot project directory via: `$ godot --headless --stagtest?`

Breakdown of command-line arguments:
- `godot` - should launch Godot Engine
- `--headless` - will run Godot headlessly. This is optional, ignore it if you want to observe your tests running.
- `--stagtest` - runs the StagTest harness
- `--stagtest?` - prints help output for using StagTest (see below)

Here is StagTest's help output.

```
StagTest - StagToolkit test harness implementation.
   flags ---
        --stagtest? - displays command output, like this
        --stagtest  - runs with StagTest mode
        --fast      - escapes on the first test failure, instead of running all tests
        --bench     - enables benchmark reports and switches default test directory to "res://test/benchmarks/"
                - benchmark times are reported in microseconds, unless otherwise specified
   arguments ---
        note: FILEPATHs can be absolute, relative, or a resource path. Resource paths are strongly advised.

        --test=FILEPATH - runs the provided scene file, or all scene files within given directory
                - if a directory, subdirectories are also run
                - organized alphabetically within each directory, running subdirectories first
                FILEPATH="res://test/scenarios/" by default (quotes optional)
        --reports=DIRECTORY - writes reports to the given directory
                - set to empty string "" for no reports
                DIRECTORY="res://test/reports/" by default (quotes optional)
        --timeout=SECONDS - forcibly ends all tests after the given amount of time, returning any collected results
                SECONDS=30.0 by default, floating-point times are valid
        --timescale=SCALE - sets the default engine time scale when not overidden by tests
                SCALE=1.0 by default, floating-point scales are valid
```

### Test Format

Example tests reside [here](godot/test/scenarios/example).

StagTest runs scene files instead of scripts, and makes full use of the SceneTree.
This means that SceneTree-related node functions/events like `_init()`, `_ready()`, `_process()` and even `_enter_tree()` and `_exit_tree()` will be executed.

StagTest will automatically look for tests inside `res://test/scenarios`, but can be pointed to either a directory or specific file with the `--test=<TestPath>` flag.

### Teardown

To finish a test, call `StagTest.teardown()` to notify StagTest to teardown the test.
**Code may still run as a test is tearing down, until it has exited the SceneTree**.

Assertions can still be run during Teardown to ensure scenes are properly cleaned up.
If the test is not skipped or failed during Teardown, the test is passed once Teardown completes.

### Overriding Test Environments

**Your startup scene will still be loaded** (and immediately after, unloaded) while setting up StagTest.
To prevent your scene from running code, return if testing is active.

```gd
func _ready():
	# DON'T switch scenes if StagTest is active
	if is_instance_valid(StagTest) and StagTest.is_active():
		return
```

StagTest will remove itself from the tree upon entering the Ready state if running in a release build, so it is necessary to check whether the singleton exists via `is_instance_valid`.

**If your game has a custom quit function** for thread safety (or other things), you can tell StagToolkit to use that instead of a traditional `get_tree().quit()`. The Callable must accept an exit code.

```gd
func quit(exit_code: int = 0):
	emit_signal("quitting")

	if Utils.is_debug():
		print("\nQUITTING, here's the orphans:")
		Node.print_orphan_nodes()

	get_tree().quit(exit_code)

func _ready():
	# Force StagToolkit to use our own teardown sequence
	StagTest.override_exit_function(quit)
```

## Methods

As tests do not run inside a testing class, you must instead reference the StagTest singleton to perform assertions.

### Test Environemnt
- `StagTest.teardown()` - **This must be called to finish any test**. Puts the test in Teardown mode, removing the scene from the tree at the end of the frame, before freeing it and passing the test. Assertions can still be during Teardown.
- `StagTest.pause(pause: bool)` - Pauses the SceneTree during a test. The SceneTree is unpaused immediately before each test, and paused while transitioning tests.
- `StagTest.time_scale(new_scale: float = 1.0)` - Sets the Engine time scale during a test. The time scale is reset immediately before each test and when transitioning tests.
- `StagTest.skip(reason: String)` - Skips the active test, ignoring further assertions and entering Teardown mode.
- `StagTest.fail(reason: String)` - Fails the active test, ignoring further assertions and entering Teardown mode.
- `StagTest.override_exit_function(new_quit: Callable)` - Overrides the runtime exit function, in case the game needs additional teardown steps.

### Assertions
Any failed assertion will immediately fail the test, entering Teardown and ignoring further assertions.
Optional messages can be included for additional context.

- `StagTest.assert_true(value: bool, message: String = "")` - Assert that a given value is true.
- `StagTest.assert_equal(a: Variant, b: Variant, message: String = "")` - Assert that two values are equal.
- `StagTest.assert_unequal(a: Variant, b: Variant, message: String = "")` - Assert that two values are NOT equal.
- `StagTest.assert_valid(a: Object, message: String = "")` - Assert that the given Object is valid.
- `StagTest.assert_approx_equal(a: Variant, b: Variant, message: String = "")` - Utilizes Godot's built-in "approx_equal" method for comparing two values. Note that this threshold value scales based on the magnitude of the values being compared, with low-precision.
- `StagTest.assert_in_delta(a: Variant, b: Variant, delta: float = 1e-5, message: String = "")` - Asserts that two variants of the same type are within a specified delta. Only some common types are supported.

### Signal Expectors

Signal Expectors are a uniqueassertion helper that can be to test signals.

- `StagTest.signal_expector(emitter: Signal, emitter_parameter_count: int, message: String = "") -> StagTest.SignalExpector`
	- Returns a SignalExpector bound to the given signal.
	- Will fail test if the provided signal is null, or fails to connect.
	- `emitter_parameter_count` must equal the number of parameters provided by the signal.
	- `message` serves as additional context.

Signal Expectors have a variety of thread-safe assertions and helper methods.

- `assert_valid(extra_context: String = "")` - Assert the emitter is not null.
- `assert_emitted(extra_context: String = "")` - Assert the Signal was emitted at least once.
- `assert_not_emitted(extra_context: String = "")` - Assert the Signal was not emitted at all.
- `assert_count(exact_call_count: int, extra_context: String = "")` - Assert the Signal was emitted exactly `exact_call_count` times.
- `block_until(threshold: int = 1, timeout_ms: int = 30000, extra_context: String = "")` - Block this thread until either the emitted count or timeout (in milliseconds) is reached. Fails the test if the timeout was reached.
- `reset()` - Resets the expector state.
- `get_count() -> int` - Returns the number of times the signal emitted.

See [`res://test/scenarios/examples/test_signals.gd`](../godot/test/scenarios/example/test_signals.gd) for example usage.


### Informational
- `StagTest.is_active()` - Returns true if StagTest is active (game launched with `--stagtest` flag)
- `StagTest.path()` - Returns the path of the active test
- `StagTest.benchmark(f: Callable, count: int, label: String, timeout: float = -1)` - Performs a timing benchmark of the Callable (with no arguments) the specified number of times, returning an analysis.
	- If timeout is greater than zero, forcibly stops benchmark after X many seconds.
	- If a test is skipped or failed during the benchmark, the benchmark exits without completing all iterations.
	- Results are always in microseconds, unless otherwise specified.
	- Use the `--bench` flag when running to output benchmark results.

## FAQ

### Why another test harness?

While the **[Godot Unit Test](https://github.com/bitwes/Gut) add-on works great for isolated unit-testing**,
I need to test messy, physics-simulated gameplay scenarios, to ensure that gameplay mechanics properly interact.

StagTest works using Godot's SceneTree like an actual game run-time,
while giving you simple tools to verify if the scene is running properly or not.

StagTest is **not intended for low-level code testing**, but it is **designed for validating high-level scenarios**.
For example: inside a game level, verify that my character spawns, that respawn points are properly registered, and that when my character walks off a cliff, they respawn at the correct respawn point.
