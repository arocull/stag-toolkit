extends EditorInspectorPlugin

var docker = preload("res://addons/stag_toolkit/ui/island_docker.tscn")
var panel: Control = null

func _can_handle(object: Object) -> bool:
	return object is IslandBuilder

func _parse_begin(object: Object) -> void:
	panel = docker.instantiate()
	
	var builder = object as IslandBuilder
	update_shapecount(builder)
	
	if builder.has_meta("volume"):
		update_volume(builder.get_meta("volume", 0.0))
	
	var bserialize: Button = panel.get_node("%btn_serialize")
	bserialize.pressed.connect(do_serialize.bind(builder))
	var bmetaclear: Button = panel.get_node("%btn_metadata")
	bmetaclear.pressed.connect(do_metaclear.bind(builder))
	var bpreview: Button = panel.get_node("%btn_preview")
	bpreview.pressed.connect(do_preview.bind(builder))
	var bbake: Button = panel.get_node("%btn_bake")
	bbake.pressed.connect(do_bake.bind(builder))
	
	add_custom_control(panel)

func update_shapecount(builder: IslandBuilder):
	panel.get_node("%shape_count").text = "{0} shapes".format([builder.shapes.size()])
func update_volume(new_volume: float):
	panel.get_node("%volume").text = "{0} m³".format([new_volume])

func do_serialize(builder: IslandBuilder):
	builder.serialize()
	update_shapecount(builder)

func do_metaclear(node: Node):
	for child in node.get_children():
		do_metaclear(child)
	node.remove_meta("edge_radius")
	node.remove_meta("hull_zscore")

func do_preview(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	find_mesh_output(builder).mesh = builder.generate_mesh_preview()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Mesh preview took ", float(t2 - t1) * 0.001, " ms")

func do_bake(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	var bake_data = builder.bake()
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: Bake took ", float(t2 - t1) * 0.001, " ms")
	var out = find_output_object(builder)
	
	if bake_data.size() <= 0:
		push_error("IslandBuilder: Bake failed")
		return
	
	var mesh: ArrayMesh = bake_data[0]
	var hulls: Array[ConvexPolygonShape3D] = bake_data[1]
	var volume: float = bake_data[2]
	
	# Update volume label
	if is_instance_valid(panel):
		update_volume(volume)
	builder.set_meta("volume", volume)
	
	# Set mesh output to baked mesh
	# TODO: Right now, this has to be baked inside of Godot due to lack of Rust support
	var importer = ImporterMesh.new()
	importer.clear()
	importer.add_surface(Mesh.PRIMITIVE_TRIANGLES, mesh.surface_get_arrays(0), [], {}, builder.island_material, "island")
	importer.generate_lods(builder.lod_normal_merge_angle, builder.lod_normal_split_angle, [])
	find_mesh_output(builder).mesh = importer.get_mesh(mesh)
	
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
		out.mass = volume * builder.density

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
