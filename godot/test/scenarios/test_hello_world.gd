extends Node3D

func _init():
	print("SCENE INITIALIZED")

func _enter_tree():
	print("SCENE ENTERED TREE")

func _ready():
	print("SCENE READY")

	StagTest.assertion(true)

	StagTest.finish()
