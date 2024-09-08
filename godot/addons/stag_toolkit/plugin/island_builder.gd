extends EditorInspectorPlugin

var docker = preload("res://addons/stag_toolkit/plugin/ui/island_docker.tscn")
var panel: Control = null
var last_builder: IslandBuilder = null

const PRECOMPUTE_REQUIRED_BUTTONS = [
	"btn_mesh_preview",
	"btn_finalize",
	"btn_mesh_bake",
	"btn_collision",
	"btn_navigation",
]

func _can_handle(object: Object) -> bool:
	if object is Node:
		var builder = fetch_builder_ancestor(object)
		return is_instance_valid(builder)
	return false

func _parse_begin(object: Object) -> void:
	panel = docker.instantiate()

	if is_instance_valid(last_builder):
		if last_builder.completed_serialize.is_connected(_on_serialize):
			last_builder.completed_serialize.disconnect(_on_serialize)
		if last_builder.completed_nets.is_connected(_on_precompute):
			last_builder.completed_nets.disconnect(_on_precompute)

	var builder: IslandBuilder = fetch_builder_ancestor(object)
	update_shapecount(builder)
	update_volume(builder)
	update_button_availability(builder)
	last_builder = builder

	builder.completed_serialize.connect(_on_serialize.bind(builder))
	builder.completed_nets.connect(_on_precompute.bind(builder))

	var bserialize: Button = panel.get_node("%btn_serialize")
	bserialize.pressed.connect(do_serialize.bind(builder))
	var bprecomp: Button = panel.get_node("%btn_precompute")
	bprecomp.pressed.connect(do_precompute.bind(builder))
	var bmetaclear: Button = panel.get_node("%btn_metadata")
	bmetaclear.pressed.connect(do_metaclear.bind(builder))

	var bmeshpreview: Button = panel.get_node("%btn_mesh_preview")
	bmeshpreview.pressed.connect(do_mesh_preview.bind(builder))
	var bmeshbake: Button = panel.get_node("%btn_mesh_bake")
	bmeshbake.pressed.connect(do_mesh_bake.bind(builder))
	var bcollision: Button = panel.get_node("%btn_collision")
	bcollision.pressed.connect(do_collision.bind(builder))
	var bnavigation: Button = panel.get_node("%btn_navigation")
	bnavigation.pressed.connect(do_navigation.bind(builder))

	var bfinalize: Button = panel.get_node("%btn_finalize")
	bfinalize.pressed.connect(do_finalize.bind(builder))

	add_custom_control(panel)

func fetch_builder_ancestor(object: Node) -> IslandBuilder:
	if object is IslandBuilder and object.has_method("net"):
		return object
	if not is_instance_valid(object.get_parent()) or object.get_tree().current_scene == self:
		return null
	return fetch_builder_ancestor(object.get_parent())

func update_button_availability(builder: IslandBuilder):
	panel.get_node("%btn_precompute").disabled = builder.get_shape_count() <= 0
	var disable_precomps = not builder.is_precomputed()
	for btn_path in PRECOMPUTE_REQUIRED_BUTTONS:
		panel.get_node("%" + btn_path).disabled = disable_precomps

func update_shapecount(builder: IslandBuilder):
	panel.get_node("%shape_count").text = "{0} shapes".format([builder.get_shape_count()])
func update_volume(builder: IslandBuilder):
	panel.get_node("%volume").text = "{0} m³".format([builder.get_volume()])

func do_serialize(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	builder.serialize()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Serialized in ", float(t2 - t1) * 0.001, " ms")
func _on_serialize(builder: IslandBuilder):
	update_shapecount(builder)
	update_button_availability(builder)
func do_precompute(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	builder.net()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Pre-computation took ", float(t2 - t1) * 0.001, " ms")
func _on_precompute(builder: IslandBuilder):
	update_volume(builder)
	update_button_availability(builder)

func do_metaclear(node: Node):
	for child in node.get_children():
		do_metaclear(child)
	node.remove_meta("edge_radius")
	node.remove_meta("hull_zscore")

func do_mesh_preview(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	find_mesh_output(builder).mesh = builder.mesh_preview()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Mesh preview took ", float(t2 - t1) * 0.001, " ms")

func do_mesh_bake(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	var mesh: ArrayMesh = builder.mesh_baked()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Mesh bake took ", float(t2 - t1) * 0.001, " ms before LOD")

	# Set mesh output to baked mesh
	# TODO: Right now, this has to be baked inside of Godot due to lack of Rust support
	var importer = ImporterMesh.new()
	importer.clear()
	importer.add_surface(Mesh.PRIMITIVE_TRIANGLES, mesh.surface_get_arrays(0), [], {}, builder.material_baked, "island")
	#importer.generate_lods(builder.lod_normal_merge_angle, builder.lod_normal_split_angle, [])
	importer.generate_lods(25, 60, [])
	mesh.clear_surfaces()
	find_mesh_output(builder).mesh = importer.get_mesh(mesh)

func do_collision(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	var hulls: Array[ConvexPolygonShape3D] = builder.collision_hulls()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Collision Hulls took ", float(t2 - t1) * 0.001, " ms before instancing")
	var out = find_output_object(builder)

	for child in out.get_children():
		if child is CollisionShape3D:
			child.free()

	# Add collision hulls
	for idx in range(0,hulls.size()):
		var item: ConvexPolygonShape3D = hulls[idx]
		var shape = CollisionShape3D.new()
		shape.shape = item
		shape.name = "collis{0}".format([idx])
		out.add_child(shape)
		shape.owner = out.get_tree().edited_scene_root

	if out is RigidBody3D:
		out.mass = builder.volume * builder.density
	if out.get_parent().has_method("set_maximum_health"):
		out.get_parent().set_maximum_health(builder.volume * builder.density_health)

func do_navigation(builder: IslandBuilder):
	var out = find_output_object(builder)
	if out.has_method("set_navigation_properties"):
		var nav_props: NavIslandProperties = builder.navigation_properties()
		out.set_navigation_properties(nav_props)

func do_finalize(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	do_mesh_bake(builder)
	do_collision(builder)
	do_navigation(builder)
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: FINALIZE ALL took ", float(t2 - t1) * 0.001, " ms")

func find_output_object(builder: IslandBuilder) -> Node:
	var out = builder.get_node(builder.output_to)
	if not is_instance_valid(out):
		return builder
	return out

func find_mesh_output(builder: IslandBuilder) -> MeshInstance3D:
	var out = find_output_object(builder)
	for child in out.get_children():
		if child is MeshInstance3D:
			return child

	var mesh = MeshInstance3D.new()
	mesh.name = 'mesh_island'
	mesh.set_layer_mask_value(1, true)
	mesh.set_layer_mask_value(2, true)
	out.add_child(mesh)
	mesh.owner = out.get_tree().edited_scene_root

	return mesh
