extends Node

## How many times to run the benchmark.
@export_range(1,100,1) var net_count: int = 100

func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.teardown.call_deferred() # End test after frame

	var isles = IslandBuilder.all_builders(get_tree())

	StagTest.assert_equal(2, isles.size(), "should have some islands to benchmark")

	for isle in isles:
		isle.serialize()
		StagTest.benchmark(isle.net, net_count,
			"surface_nets on {0}".format([isle.get_parent_node_3d().name]))
