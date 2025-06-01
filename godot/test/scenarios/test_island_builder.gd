extends Node3D

@onready var builder: IslandBuilder = $IslandBuilder

@export var nav_properties: NavIslandProperties = null

func teardown():
	StagTest.teardown.call_deferred()

func _ready():
	# Complete test after two frames
	teardown.call_deferred()

	StagTest.assert_valid(builder, "IslandBuilder should be valid")

	# Serialize IslandBuilder to get shapes
	builder.serialize()
	StagTest.assert_equal(builder.get_shape_count(), 4, "should serialize all visible CSG nodes")

	# Calculate surface nets
	builder.net()
	StagTest.assert_true(builder.get_volume() > 1.0, "mesh should have significant amount of volume")

	var builder_aabb: AABB = builder.estimate_aabb()
	StagTest.assert_true(builder_aabb.has_volume(), "IslandBuilder's AABB estimate should have volume")

	# Get preview and finalized meshes
	var preview_mesh: ArrayMesh = builder.generate_preview_mesh(null)
	var baked_mesh: ArrayMesh = builder.generate_baked_mesh()

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
		preload("res://3d/islandbuilder/materials/mat_testing_preview.tres"),
		preview_mesh.surface_get_material(0),
		"preview mesh should have correct surface material")

	StagTest.assert_equal(
		preload("res://3d/islandbuilder/materials/mat_testing_baked.tres"),
		baked_mesh.surface_get_material(0),
		"baked mesh should have correct surface material")

	# Generating collision hulls
	var hulls = builder.generate_collision_hulls()
	StagTest.assert_equal(2, hulls.size(), "should be exactly 2 collision hulls")

	# Fetching target
	StagTest.assert_valid(builder.target(), "builder target should always be valid")
	StagTest.assert_equal($body, builder.target(), "should target correct node")

	# Fetching + creating target mesh
	StagTest.assert_valid(builder.target_mesh(), "target mesh should be instantiated")
	StagTest.assert_true($body.is_ancestor_of(builder.target_mesh()), "target mesh should be child of builder target")
	StagTest.assert_true(builder.target_mesh().get_layer_mask_value(3), "target mesh should be on layer 3")


	# Generating NavIslandProperties (must be serialized first)
	nav_properties = null
	var props = builder.generate_navigation_properties()
	StagTest.assert_valid(props, "NavIslandProperties should be valid")
	StagTest.assert_true(props.aabb.size.length() > 0.1, "NavIslandProperties should have valid data")

	# Applying NavIslandProperties. Note: this will fail in-editor if the target is not a tool script
	builder.apply_navigation_properties(props)
	StagTest.assert_valid(nav_properties, "NavIslandProperties should have been applied")

	# Destroying bakes
	builder.target_mesh().mesh = builder.generate_preview_mesh(null)
	StagTest.assert_valid(builder.target_mesh(), "target mesh should be instantiated from preview")
	builder.destroy_bakes()
	StagTest.assert_valid($body/mesh_island, "target mesh should still exist after destroying bakes")
	StagTest.assert_equal(null, builder.target_mesh().mesh, "target mesh should have mesh asset cleared")

	# Fetching all builders
	var builders = IslandBuilder.all_builders(get_tree())
	StagTest.assert_equal(1, builders.size(), "should have retrieved 1 IslandBuilder")
	StagTest.assert_valid(builders[0], "retrieved IslandBuilder should be valid")
	StagTest.assert_equal(builder, builders[0], "retrieved IslandBuilder")

	# Destroying ALL bakes
	IslandBuilder.all_destroy_bakes(get_tree())
	StagTest.assert_valid($body/mesh_island, "target mesh should still exist after destroying ALL bakes")
	StagTest.assert_equal(null, builder.target_mesh().mesh,
		"target mesh should have mesh asset cleared after destroying ALL bakes")

	# Baking ALL islands
	IslandBuilder.all_bake(get_tree())
	await builder.applied_build_data # Wait for build data to be applied
	StagTest.assert_equal(3, builder.target().get_child_count(),
		"IslandBuilder.all_bake should have built collision for the target")
	StagTest.assert_valid(builder.target_mesh().mesh, "IslandBuilder.all_bake should have created a mesh resource")

func set_navigation_properties(props: NavIslandProperties):
	nav_properties = props
