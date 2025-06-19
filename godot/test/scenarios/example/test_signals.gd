extends Node3D

signal test_signal(arg1: int)

# Called when the node enters the scene tree for the first time.
func _ready():
	# Create a SignalExpector
	var expector := StagTest.signal_expector(
		test_signal, # Signal to listen for
		1, # Number of arguments in signal (required for Godot to discard them)
		"this is additional context" # Optional additional context on failure
	)

	# Assert that the signal was NOT emitted
	expector.assert_not_emitted("signal should not be emitted during expector creation")


	## Asserting that the signal emitted, when it hasn't, will fail the test.
	# expector.assert_emitted("this will fail")


	test_signal.emit(0)
	expector.assert_emitted("signal should have emitted")
	expector.assert_count(1, "signal should have only emitted once")


	expector.reset() # Reset the signal expector
	expector.assert_not_emitted("signal expector should be reset")


	# Call the signal multiple times and look for an exact count
	for i in range(0, 10):
		test_signal.emit(i)
	expector.assert_count(10, "signal should have emitted ten times")


	# Signal Expector is thread-safe and can be used to block for co-routines
	expector.reset()
	print("Starting worker threads!")
	var worker_group := WorkerThreadPool.add_group_task(threaded_call.bind(test_signal), 5, -1, true, "Signal Expector test")

	# Block thread until 5 calls have been made, or fail if 10 seconds have passed
	print("Starting waiting!")
	expector.block_until(5, 10000, "multi-threading example")
	# should see output "Firing 4" "Firing 3" ...
	print("Finished waiting!")

	expector.assert_count(5, "sanity check that multi-threading example worked")
	WorkerThreadPool.wait_for_group_task_completion(worker_group) # Collect threads


	# Finish test
	StagTest.teardown()

func threaded_call(worker_idx: int, emitter: Signal):
	Thread.set_thread_safety_checks_enabled(false) # Go unhinged

	OS.delay_msec(10) # Wait a bit

	print("Firing {0}".format([worker_idx]))
	emitter.emit(worker_idx) # Fire the event
