extends Node3D

@onready var builder: IslandBuilder = $IslandBuilder

func _ready():
	# Complete test after frame
	StagTest.teardown.call_deferred()

	StagTest.assert_valid(builder, "IslandBuilder should be valid")

	# Serialize IslandBuilder to get shapes
	builder.serialize()
	StagTest.assert_equal(builder.get_shape_count(), 3, "should serialize all visible CSG nodes")

	# Calculate surface nets
	builder.net()
	StagTest.assert_true(builder.get_volume() > 1.0, "mesh should have significant amount of volume")

	var builder_aabb: AABB = builder.estimate_aabb()
	StagTest.assert_true(builder_aabb.has_volume(), "IslandBuilder's AABB estimate should have volume")

	# Get preview and finalized meshes
	var preview_mesh: ArrayMesh = builder.mesh_preview(null)
	var baked_mesh: ArrayMesh = builder.mesh_baked()

	StagTest.assert_valid(preview_mesh, "preview mesh should be valid")
	StagTest.assert_valid(baked_mesh, "baked mesh should be valid")

	# Fetch mesh AABB's
	var preview_aabb: AABB = preview_mesh.get_aabb()
	var baked_aabb: AABB = baked_mesh.get_aabb()

	StagTest.assert_equal(preview_mesh.get_surface_count(), 1, "preview mesh should have 1 surface")
	StagTest.assert_equal(baked_mesh.get_surface_count(), 1, "baked mesh should have 1 surface")

	StagTest.assert_true(preview_aabb.has_volume(), "preview mesh should have volume")
	StagTest.assert_true(baked_aabb.has_volume(), "baked mesh should have volume")

	StagTest.assert_equal(
		preload("res://3d/islandbuilder/materials/mat_island_sandy_nobake.tres"),
		preview_mesh.surface_get_material(0),
		"preview mesh should have correct surface material")

	StagTest.assert_equal(
		preload("res://3d/islandbuilder/materials/mat_island_sandy.tres"),
		baked_mesh.surface_get_material(0),
		"baked mesh should have correct surface material")

	var hulls = builder.collision_hulls()

	StagTest.assert_equal(1, hulls.size(), "should be exactly 1 collision hull")
