extends Node

## How many times to run the benchmark.
@export_range(1,100,1) var bake_count: int = 3

func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.skip("Figure out how to bring builder data back to main thread without deferring")
	# run_bench()
	StagTest.teardown()

func run_bench():
	var builders := IslandBuilder.all_builders(get_tree())
	var expectors: Array[StagTest.SignalExpector] = []

	for builder in builders:
		expectors.append(StagTest.signal_expector(builder.applied_build_data, 0, "Island {0}".format([builder.name])))

	StagTest.benchmark(all_bake.bind(expectors), bake_count, "bake all islands")

func all_bake(expectors: Array[StagTest.SignalExpector]):
	IslandBuilder.all_bake(get_tree()) # Bake all Islands
	for expector in expectors: # Wait for threads to join main thread
		expector.block_until(1)
