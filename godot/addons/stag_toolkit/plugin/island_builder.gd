extends EditorInspectorPlugin

const TWEAK_TIMER_THRESHOLD = 1000 # After 1 second idle time, update preview
const TWEAK_TIMEOUT_THRESHOLD = 10000 # After 10 seconds, reset the queue

const PRECOMPUTE_REQUIRED_BUTTONS = [
	"btn_mesh_preview",
	"btn_finalize",
	"btn_mesh_bake",
	"btn_collision",
	"btn_navigation",
]

var docker = preload("res://addons/stag_toolkit/plugin/ui/island_docker.tscn")
var panel: Control = null
var last_builder: IslandBuilder = null
var realtime_enabled: bool = false
var csg_linting: bool = false
var transforms: Dictionary

func _can_handle(object: Object) -> bool:
	if object is Node:
		var builder = fetch_builder_ancestor(object)
		if is_instance_valid(builder):
			return true
	realtime_enabled = false
	return false

func _parse_begin(object: Object) -> void:
	panel = docker.instantiate()

	var builder: IslandBuilder = fetch_builder_ancestor(object)
	update_shapecount(builder)
	update_volume(builder)
	update_button_availability(builder)

	var this_is_previous: bool = last_builder == builder
	if not this_is_previous:
		realtime_enabled = false # Disable realtime meshing when re-entering an island builder
		transforms.clear()

		if is_instance_valid(last_builder):
			unbind_realtime(last_builder)
			if last_builder.completed_serialize.is_connected(_on_serialize):
				last_builder.completed_serialize.disconnect(_on_serialize)
			if last_builder.completed_nets.is_connected(_on_precompute):
				last_builder.completed_nets.disconnect(_on_precompute)
			if last_builder.get_tree().process_frame.is_connected(_check_transforms):
				last_builder.get_tree().process_frame.disconnect(_check_transforms)

		builder.completed_serialize.connect(_on_serialize.bind(builder), CONNECT_DEFERRED)
		builder.completed_nets.connect(_on_precompute.bind(builder), CONNECT_DEFERRED)

		bind_realtime(builder)

		if not builder.get_tree().process_frame.is_connected(_check_transforms):
			builder.get_tree().process_frame.connect(_check_transforms)

	last_builder = builder

	var bserialize: Button = panel.get_node("%btn_serialize")
	bserialize.pressed.connect(do_serialize.bind(builder))
	var bprecomp: Button = panel.get_node("%btn_precompute")
	bprecomp.pressed.connect(do_precompute.bind(builder))
	var bmetaclear: Button = panel.get_node("%btn_metadata")
	bmetaclear.pressed.connect(do_metaclear.bind(builder))

	var bmeshpreview: Button = panel.get_node("%btn_mesh_preview")
	bmeshpreview.pressed.connect(do_mesh_preview.bind(builder))
	var bfinalize: Button = panel.get_node("%btn_finalize")
	bfinalize.pressed.connect(do_finalize.bind(builder))
	var bdestroy: Button = panel.get_node("%btn_destroy")
	bdestroy.pressed.connect(do_destroy.bind(builder))

	var bmeshbake: Button = panel.get_node("%btn_mesh_bake")
	bmeshbake.pressed.connect(do_mesh_bake.bind(builder))
	var bcollision: Button = panel.get_node("%btn_collision")
	bcollision.pressed.connect(do_collision.bind(builder))
	var bnavigation: Button = panel.get_node("%btn_navigation")
	bnavigation.pressed.connect(do_navigation.bind(builder))

	var trealtime: CheckBox = panel.get_node("%toggle_realtime")
	trealtime.button_pressed = realtime_enabled
	trealtime.toggled.connect(_realtime_toggled)

	var tcsglint: CheckBox = panel.get_node("%toggle_csg_linter")
	tcsglint.button_pressed = csg_linting
	tcsglint.toggled.connect(_csg_linter)

	if csg_linting:
		lint_node(object)

	if not EditorInterface.get_inspector().property_edited.is_connected(on_property_change):
		EditorInterface.get_inspector().property_edited.connect(on_property_change)

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

# Destroys any binary IslandBuilder data for safe git saving
func do_destroy(node: IslandBuilder):
	for child in find_output_object(node).get_children():
		if child is CollisionShape3D:
			child.queue_free() # Remove collision shapes
		elif child is MeshInstance3D:
			child.mesh = null # Unlinks the mesh data

func do_mesh_preview(builder: IslandBuilder):
	var t1 = Time.get_ticks_usec()
	find_mesh_output(builder).mesh = builder.mesh_preview(null)
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
		out.mass = builder.get_volume() * builder.gameplay_density
		out.axis_lock_angular_x = true
		out.axis_lock_angular_z = true
		out.axis_lock_linear_y = true
	if out.get_parent().has_method("set_maximum_health"):
		out.get_parent().set_maximum_health(builder.get_volume() * builder.gameplay_health_density)

func do_navigation(builder: IslandBuilder):
	var out = find_output_object(builder)
	if out.has_method("set_navigation_properties"):
		var nav_props: NavIslandProperties = builder.navigation_properties()
		out.set_navigation_properties(nav_props)

func do_finalize(builder: IslandBuilder):
	realtime_enabled = false
	panel.get_node("%toggle_realtime").button_pressed = false
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
	mesh.set_layer_mask_value(2, false)
	mesh.set_layer_mask_value(3, true)
	out.add_child(mesh)
	mesh.owner = out.get_tree().edited_scene_root

	return mesh


# CSG LINTING
const LINT_PREFIXES = [
	"UNION_",
	"INTERSECT_",
	"SUBTRACT_",
]

func _csg_linter(new_state: bool):
	csg_linting = new_state
	if csg_linting and is_instance_valid(last_builder):
		lint_node_recursive(last_builder)

# Lints the given string with the according suffix
func lint_name(name: String, operation: int, suffix: String):
	# Generic names should always be replaced
	var tailor: bool = name.begins_with("CSG")

	# If the item has a prefix
	for item in LINT_PREFIXES:
		if name.begins_with(item):
			tailor = true
			suffix = name.replace(item, "") # Strip prefix and retain name
			break

	if not tailor:
		return

	match operation:
		0: #OPERATION_UNION:
			name = "UNION_"
		1: #OPERATION_INTERSECTION:
			name = "INTERSECT_"
		2: #OPERATION_SUBTRACT:
			name = "SUBTRACT_"
	return name + suffix

# Returns a material to represent the given CSG operation
func lint_material(operation: int):
	match operation:
		0: #OPERATION_UNION:
			return load("res://addons/stag_toolkit/utils/shaders/matdebug_csg_union.tres")
		1: #OPERATION_INTERSECTION:
			return load("res://addons/stag_toolkit/utils/shaders/matdebug_csg_intersect.tres")
		2: #OPERATION_SUBTRACT:
			return load("res://addons/stag_toolkit/utils/shaders/matdebug_csg_subtract.tres")
		_:
			return load("res://addons/stag_toolkit/utils/shaders/matdebug_overdraw.tres")

# Performs CSG linting on the given node for better readability
func lint_node(node: Node):
	if is_instance_valid(last_builder) and last_builder.is_ancestor_of(node):
		if node is CSGBox3D:
			node.name = lint_name(node.name, node.operation, "box")
			node.material_override = lint_material(node.operation)
		if node is CSGSphere3D:
			node.name = lint_name(node.name, node.operation, "sphere")
			node.material_override = lint_material(node.operation)

func lint_node_recursive(node: Node):
	for child in node.get_children():
		lint_node_recursive(child)
	lint_node(node)

# REALTIME PREVIEW
var realtime_thread: Thread
var realtime_queued: bool = false
var realtime_last_update: int = -1
var realtime_dirty: bool = false

# Initializes the realtime thread
func thread_init() -> void:
	realtime_thread = Thread.new()
	EditorInterface.get_inspector().property_edited.connect(on_property_change)
# Deinitializes the realtime thread
func thread_deinit() -> void:
	if realtime_thread.is_started():
		realtime_thread.wait_to_finish()
	realtime_thread = null

func _realtime_toggled(new_state: bool) -> void:
	realtime_enabled = new_state
	if realtime_thread.is_started():
		realtime_thread.wait_to_finish()

# Unbind tree and property updates from mesh generation
func unbind_realtime(node: Node) -> void:
	if node.child_entered_tree.is_connected(on_child_added):
		node.child_entered_tree.disconnect(on_child_added)
	if node.child_exiting_tree.is_connected(on_child_removed):
		node.child_exiting_tree.disconnect(on_child_removed)
	if node.child_order_changed.is_connected(update_realtime_preview):
		node.child_order_changed.disconnect(update_realtime_preview)
	if node.has_signal("visibility_changed"):
		if node.visibility_changed.is_connected(update_realtime_preview):
			node.visibility_changed.disconnect(update_realtime_preview)
	for child in node.get_children():
		unbind_realtime(child)
# Bind tree and property updates to mesh regeneration
func bind_realtime(node: Node, top_level: bool = false) -> void:
	if top_level:
		node.child_order_changed.connect(update_realtime_preview)
		node.child_entered_tree.connect(on_child_added)
		node.child_exiting_tree.connect(on_child_removed)
	if node.has_signal("visibility_changed"):
		node.visibility_changed.connect(update_realtime_preview)

	for child in node.get_children():
		bind_realtime(child, false)

func on_property_change(property: String):
	if csg_linting and property == "operation":
		var obj = EditorInterface.get_inspector().get_edited_object()
		if is_instance_valid(obj) and obj is Node:
			lint_node(obj)

	if realtime_enabled and is_instance_valid(last_builder):
		update_realtime_preview()
func on_child_added(new_child: Node):
	bind_realtime(new_child, false)

	if csg_linting:
		lint_node(new_child)
func on_child_removed(new_child: Node):
	unbind_realtime(new_child)

func _check_transforms() -> void:
	if realtime_enabled and is_instance_valid(last_builder):
		_check_transforms_internal(last_builder)

		var t = Time.get_ticks_msec()
		# If we have new changes, but haven't updated our generation in a while, do a clean pass to ensure we're at final
		if realtime_dirty and not realtime_queued:
			if t > realtime_last_update + TWEAK_TIMER_THRESHOLD:
				update_realtime_preview(false)
		# If somehow our thread failed, reset our queue status
		if realtime_queued and t > realtime_last_update + TWEAK_TIMEOUT_THRESHOLD:
			realtime_queued = false
func _check_transforms_internal(node: Node) -> void:
	if node is Node3D:
		var old_transform: Transform3D = transforms.get(node.get_instance_id(), node.transform)
		if not old_transform.is_equal_approx(node.transform):
			update_realtime_preview()
			# print("transform updated, requesting realtime preview")
			#return # We don't need to check the rest of the transforms!
		transforms[node.get_instance_id()] = node.transform

	for child in node.get_children():
		_check_transforms_internal(child)

# Called if the IslandBuilder tree changed somehow
func update_realtime_preview(dirty: bool = true):
	if not realtime_enabled: return

	realtime_dirty = dirty

	if realtime_queued: return
	realtime_queued = true
	_update_realtime_preview_deferred.call_deferred()

func _update_realtime_preview_deferred():
	# print("update realtime preview event")
	last_builder.serialize()
	if realtime_thread.is_started():
		# print("Awaiting thread")
		realtime_thread.wait_to_finish()
	realtime_thread.start(_realtime_preview.bind(last_builder, _realtime_preview_finish.bind(last_builder)))

func _realtime_preview(builder: IslandBuilder, on_finish: Callable) -> void:
	Thread.set_thread_safety_checks_enabled(false)
	# print("doing nets")
	if builder.net(): return # Buffer was empty
	# print("doing mesh preview")
	on_finish.call_deferred(builder.mesh_preview(null))
func _realtime_preview_finish(new_mesh: ArrayMesh, builder: IslandBuilder) -> void:
	# print("applying realtime")
	find_mesh_output(builder).mesh = new_mesh
	realtime_queued = false
	realtime_last_update = Time.get_ticks_msec()
