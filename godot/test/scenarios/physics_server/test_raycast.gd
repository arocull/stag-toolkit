extends Node3D

@export var visualize_mode = false

@onready var server: StagPhysicsServer = $StagPhysicsServer

func _ready() -> void:
	if not visualize_mode:
		StagTest.teardown.call_deferred()

	# First, register physics bodies

	# Exists on just one channel
	StagTest.assert_equal(1, register_body($body_origin, 1), "body IDs should be deterministic")

	# Exists on all channels
	StagTest.assert_equal(2, register_body($body_below_origin, StagUtils.INT32_MAX), "body IDs should be deterministic")
	StagTest.assert_equal(3, register_body($body_transformed, StagUtils.INT32_MAX), "body IDs should be deterministic")

	var miss_everything := server.raycast(Vector3(0.0, 10.0, 0.0), Vector3.UP, 100.0, false, 1)
	StagTest.assert_true(miss_everything.is_empty(),
		"should be able to completely miss bodies, got result {0}".format([miss_everything]))
	# TODO: enabling hit_backfaces causes ray to still collide

	# Raycast down to hit origin cube
	var hit_origin := server.raycast(Vector3(0.0, 10.0, 0.0), Vector3.DOWN, 100.0, false, 1)
	StagTest.assert_true(hit_origin.has("point"), "hit_origin should have collided")
	place_debug($debug_1, hit_origin.get("point"), hit_origin.get("normal"))

	# Use a different layer to hit below the origin cube
	var hit_below_origin := server.raycast(Vector3(0.0, 10.0, 0.0), Vector3.DOWN, 100.0, false, 2)
	StagTest.assert_true(hit_below_origin.has("point"), "hit_below_origin should have collided")
	place_debug($debug_2, hit_below_origin.get("point"), hit_below_origin.get("normal"))

	# Raycast down to hit a rotated body
	var hit_transformed := server.raycast(Vector3(0.0, 10.0, 0.0), Vector3.DOWN, 100.0, false, 1)
	StagTest.assert_true(hit_transformed.has("point"), "hit_transformed should have collided")
	place_debug($debug_3, hit_transformed.get("point"), hit_transformed.get("normal"))

	print(miss_everything)
	print(hit_origin)
	print(hit_below_origin)
	print(hit_transformed)

	print("Finished!")

func register_body(node: Node3D, channels: int) -> int:
	# Register a new physics body
	var id := server.register_body([node.get_node("collision").shape], 1.0, channels, channels)

	# Assert that it was properly created
	StagTest.assert_unequal(0, id, "body ID should never be zero")

	# Update transform for the physics body
	server.set_body_state(id, node.global_transform, Vector3.ZERO, Vector3.ZERO)

	return id

func place_debug(node: Node3D, pos: Vector3, _norm: Vector3) -> void:
	node.global_position = pos
	# TODO: draw normal for visualization
	node.visible = true
