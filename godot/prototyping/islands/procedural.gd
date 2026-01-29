extends Node

@export_range(1,30,1,"or_greater") var shape_count: int = 5
@export var settings: IslandBuilderSettings
@export var tweaks: IslandBuilderSettingsTweaks

@export var size_range: Vector2 = Vector2(1.0, 10.0)
@export var distance_range: Vector2 = Vector2(0.5,5.0)
@export var random_seed: int = 0

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var t_start: int = Time.get_ticks_msec()
	print("Start")

	var builder := IslandBuilder.new()
	builder.settings = settings
	builder.tweaks = tweaks

	var rng := RandomNumberGenerator.new()
	rng.seed = random_seed

	var t_shapes := Time.get_ticks_msec()
	for i in range(shape_count):
		var shape := CSGBox3D.new()
		shape.size = Vector3(
			rng.randf_range(size_range.x, size_range.y),
			rng.randf_range(size_range.x, size_range.y),
			rng.randf_range(size_range.x, size_range.y),
		)
		shape.position = Vector3(
			rng.randf() - 0.5,
			rng.randf() - 0.5,
			rng.randf() - 0.5,
		).normalized() * rng.randf_range(distance_range.x, distance_range.y)
		shape.basis = Basis(Quaternion(rng.randf() - 0.5, rng.randf() - 0.5, rng.randf() - 0.5, rng.randf() - 0.5).normalized())
		builder.add_child(shape)
	builder.serialize()
	$world.add_child(builder)
	builder.output_to = builder.get_path_to($world/result)

	var t_gen := Time.get_ticks_msec()
	builder.build()

	var t_cleanup := Time.get_ticks_msec()
	$world.remove_child(builder)
	builder.queue_free()

	var t_end := Time.get_ticks_msec()
	print("Finish: instance {0}ms\t shapes {1}ms\t generation {2}ms\t cleanup {3}ms".format([
		t_shapes - t_start, t_gen - t_shapes, t_cleanup - t_gen, t_end - t_cleanup]))
