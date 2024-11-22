extends Node3D

func _init():
	print("I'm INITIALIZED!")
	StagTest.assert_true(true, "this is optional context")

func _enter_tree():
	print("I'm ENTERING TREE!")
	StagTest.assert_equal(1, 1, "ensure two variants are equal")

func _ready():
	print("I'm READY!")
	StagTest.assert_unequal(1, 2, "ensure two variants are NOT equal")

	StagTest.assert_valid(self, "ensure that the provided Object is valid")

func _process(delta):
	print("I'm PROCESSING for {0} seconds!".format([delta]))
	StagTest.teardown()

func _exit_tree():
	print("I'm even EXITING THE TREE!")
