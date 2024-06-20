extends EditorInspectorPlugin

var docker = preload("res://addons/stag_toolkit/ui/island_docker.tscn")
var panel: Control = null

func _can_handle(object: Object) -> bool:
	return object is IslandBuilder

func _parse_begin(object: Object) -> void:
	panel = docker.instantiate()
	
	var builder = object as IslandBuilder
	update_shapecount(builder)
	
	var bserialize: Button = panel.get_node("%btn_serialize")
	bserialize.pressed.connect(do_serialize.bind(builder))
	var bpreview: Button = panel.get_node("%btn_preview")
	bpreview.pressed.connect(do_preview.bind(builder))
	var bbake: Button = panel.get_node("%btn_bake")
	bbake.pressed.connect(do_bake.bind(builder))
	
	add_custom_control(panel)

func update_shapecount(builder: IslandBuilder):
	panel.get_node("%shape_count").text = "{0} shapes".format([builder.shapes.size()])

func do_serialize(builder: IslandBuilder):
	builder.serialize()
	update_shapecount(builder)

func do_preview(builder: IslandBuilder):
	find_mesh_output(builder).mesh = builder.generate_mesh()

func do_bake(builder: IslandBuilder):
	var bind = _on_bake_complete.bind(builder)
	builder.generated_mesh.connect(bind, CONNECT_ONE_SHOT | CONNECT_DEFERRED)
	
	builder.generate_mesh() # Perform mesh generation
	
	if builder.generated_mesh.is_connected(bind): # Disconnect event
		builder.generated_mesh.disconnect(bind)

func _on_bake_complete(island_mesh: ArrayMesh, island_pts: PackedVector3Array, builder: IslandBuilder):
	var mesh = find_mesh_output(builder)
	mesh.mesh = island_mesh
	
	var hulls = builder.generate_collision(builder.get_node(builder.output_to), island_pts)
	var out = find_output_object(builder)
	
	for idx in range(0,hulls.size()):
		var item: ConvexPolygonShape3D = hulls[idx]
		var shape = CollisionShape3D.new()
		shape.shape = item
		shape.name = "collis{0}".format([idx])
		out.add_child(shape)
		shape.owner = out.get_tree().edited_scene_root

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
