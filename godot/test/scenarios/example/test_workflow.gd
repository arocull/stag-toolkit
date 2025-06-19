extends Node3D

func _init():
	print("Starting a test that's doomed to fail!")
	StagTest.skip("failing test that we won't fix as it's an example")

func _ready():
	StagTest.assert_true(false)
	StagTest.assert_equal(1, 2)
	StagTest.assert_unequal(1, 1)

	StagTest.teardown()

func _exit_tree():
	StagTest.fail("idk")
	print("Finishing doomed test!")
