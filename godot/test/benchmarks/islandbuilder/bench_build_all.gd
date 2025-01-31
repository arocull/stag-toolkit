extends Node

## How many times to run the benchmark.
@export_range(1,100,1) var bake_count: int = 3

func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.teardown.call_deferred() # End test after frame
	# StagTest.skip("Unused")
	StagTest.benchmark(bake_all, bake_count, "bake all islands")

func bake_all():
	var isles = IslandBuilder.all_builders(get_tree())
	for isle in isles:
		isle.serialize()
	var group_id = WorkerThreadPool.add_group_task(_bake_single.bind(isles), isles.size())
	WorkerThreadPool.wait_for_group_task_completion(group_id)

func _bake_single(idx: int, isles: Array[IslandBuilder]):
	IslandBuilder.internal_bake_single(idx, isles)
