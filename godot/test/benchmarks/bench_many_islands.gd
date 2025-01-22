extends Node3D

@export_range(1,100,1) var bake_count: int = 5

# Called when the node enters the scene tree for the first time.
func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.teardown.call_deferred() # End test after frame
	StagTest.skip("currently inefficient")
	# StagTest.benchmark(bake_all, bake_count, "bake all")

func bake_all():
	IslandBuilder.all_bake(get_tree())
