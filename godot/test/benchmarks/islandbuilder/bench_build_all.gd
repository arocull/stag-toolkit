extends Node

## How many times to run the benchmark.
@export_range(1,100,1) var bake_count: int = 3

func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.teardown.call_deferred() # End test after frame
	# StagTest.skip("Unused")
	StagTest.benchmark(IslandBuilder.all_bake.bind(get_tree()) , bake_count, "bake all islands")
