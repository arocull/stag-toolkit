extends Node3D

func _init():
	print("Starting a test that's doomed to fail!")
	StagTest.skip("failing test that we won't fix as it's an example")

func _ready():
	# StagTest will fail on bad assertions
	StagTest.assert_true(false)
	StagTest.assert_equal(1, 2)
	StagTest.assert_unequal(1, 1)
	StagTest.assert_valid(null)

	# StagTest will pick up on pushed errors
	push_error("pushing errors will fail the test")
	# ...but warnings are ignored.
	push_warning("this is okay!")

	# Additionally, script errors will also be caught
	var test: Object = null
	test.free()

	# This includes Godot's built-in assertions
	assert(false, "failed assertions will fail the test")

	StagTest.teardown()

func _exit_tree():
	StagTest.fail("idk")
	print("Finishing doomed test!")
