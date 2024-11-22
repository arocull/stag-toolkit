extends Node3D

signal test_signal(arg1: int)

# Called when the node enters the scene tree for the first time.
func _ready():
	var test_signal_expector = StagTest.signal_expector(
		# Signal to listen for
		test_signal,

		# Method to connect signal to (that has enough supporting args, plus expector call)
		func (_arg1, item: Callable): item.call(),

		# Optional additional context on failure
		"this is additional context"
	)

	# print("Expect the signal to be called since creating the Expector.")
	# Test will fail here if ran.
	# test_signal_expector.call(true)

	print("Expect the signal to NOT be called since creating the Expector.")
	# This should be okay.
	test_signal_expector.call(false)

	print("Emit the event.")
	test_signal.emit(0)

	print("Expect the signal to have been called since creating the Expector.")
	test_signal_expector.call(true)

	# Finish test.
	StagTest.teardown()
