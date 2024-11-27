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
- `--headless` - will run Godot headlessly. This is optional, in case you want to observe your tests running.
- `--stagtest` - runs the StagTest harness
- `--stagtest?` - prints help output for using StagTest (lists extra CLI args)

### Test Format

Example tests reside [here](godot/test).

StagTest runs scene files instead of scripts, and makes full use of the SceneTree.
This means that SceneTree-related node functions/events like `_init()`, `_ready()`, `_process()` and even `_enter_tree()` and `_exit_tree()` will be executed.

StagTest will automatically look for tests inside `res://test/scenarios`, but can be pointed to either a directory or specific file with the `--test=<TestPath>` flag.

### Teardown

To finish a test, call `StagTest.teardown()` to notify StagTest to teardown the test.
**Code may still run as a test is tearing down, until it has exited the SceneTree**.

Assertions can still be run during Teardown to ensure scenes are properly cleaned up.
If the test is not skipped or failed during Teardown, the test is passed once Teardown completes.

### Overriding Test Environments

**Your startup scene will still be loaded** (and immediately after, unloaded) when initializing StagTest.
To prevent it from running code, return if testing is active.

```gd
func _ready():
	# DON'T switch scenes if StagTest is active
	if is_instance_valid(StagTest) and StagTest.is_active():
		return
```

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
- `StagTest.assert_approx_equal(a: Variant, b: Variant, threshold: float = 1e-5, message: String = "")` - Asserts that two variants of the same type are approximately equal. Threshold is currently only implemented for scalar values.

<br/>

Signal Expectors are a unique form of assertion that can be to test signals:
- `StagTest.signal_expector(sig: Signal, to_connect: Callable, message: String = "") -> Callable`
    - Returns a callable that ensures the given signal was (or was not) emitted.
    - `to_connect` should be a function with as many arguments as the signal emits, plus a callable argument that is called within the function.
    - Must be created before the signal is called.
    - To run the assertion, call the returned callable, passing `true` (signal SHOULD have been emitted) or `false` (signal should NOT have been emitted) depending on desired outcome.
    - See [`res://test/scenarios/signals_test.gd`](../../godot/test/scenarios/signals_test.gd) for example.


### Informational
- `StagTest.is_active()` - Returns true if StagTest is active (game launched with `--stagtest` flag)
- `StagTest.path()` - Returns the path of the active test

## FAQ

### Why another test harness?

While the **[Godot Unit Test](https://github.com/bitwes/Gut) add-on works great for isolated unit-testing**,
I need to test messy, physics-simulated gameplay scenarios, to ensure that gameplay mechanics properly interact.

StagTest works using Godot's SceneTree like an actual game run-time,
while giving you simple tools to verify if the scene is running properly or not.

StagTest is **not intended for low-level code testing**, but it is **designed for validating high-level scenarios**.
For example: inside a game level, verify that my character spawns, that respawn points are properly registered, and that when my character walks off a cliff, they respawn at the correct respawn point.
