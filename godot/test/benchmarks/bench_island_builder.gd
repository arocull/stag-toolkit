extends Node3D

@onready var builder: IslandBuilder = $IslandBuilder
@onready var preview_mesh: ArrayMesh

func _ready():
	StagTest.pause(true) # Pause scene tree
	StagTest.teardown.call_deferred() # End test after frame

	StagTest.benchmark(bench_serialize, 10000, "serialize")
	StagTest.benchmark(bench_net, 1000, "surface nets", 10.0) # Perform surface nets for 10 seconds
	StagTest.benchmark(bench_mesh_preview, 1000, "mesh preview", 10.0)
	StagTest.benchmark(bench_mesh_preview_preallocated, 1000, "prealloc preview", 10.0)
	StagTest.benchmark(bench_mesh_baked, 1000, "mesh baked", 10.0)
	StagTest.benchmark(bench_collision_hulls, 1000, "collision hulls", 10.0)

func bench_serialize():
	builder.serialize()

func bench_net():
	builder.net()

func bench_mesh_preview():
	builder.mesh_preview(null)

func bench_mesh_preview_preallocated():
	builder.mesh_preview(preview_mesh)

func bench_mesh_baked():
	builder.mesh_baked()

func bench_collision_hulls():
	builder.collision_hulls()