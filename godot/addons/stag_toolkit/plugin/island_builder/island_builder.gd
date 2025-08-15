@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
extends EditorInspectorPlugin

const TWEAK_TIMER_THRESHOLD = 1000 ## After 1 second idle time, update preview
const TWEAK_TIMEOUT_THRESHOLD = 10000 ## After 10 seconds, reset the queue

const PRECOMPUTE_REQUIRED_BUTTONS = [
	"btn_mesh_preview",
	"btn_mesh_bake",
	"btn_collision",
	"btn_navigation",
]

var docker = load("res://addons/stag_toolkit/plugin/island_builder/island_docker.tscn")
var panel: Control = null
var last_builder: IslandBuilder = null
var csg_linting: bool = false
var allow_destructive: bool = false
var transforms: Dictionary

var buttons: Array[Dictionary] = [
	{
		"id": "%serialize",
		"connect": do_serialize,
		"bind": true,
	},
	{
		"id": "%mesh_preview",
		"connect": do_mesh_preview,
		"bind": true,
	},
	{
		"id": "%mesh_bake",
		"connect": do_mesh_bake,
		"bind": true,
	},
	{
		"id": "%collision",
		"connect": do_collision,
		"bind": true,
	},
	{
		"id": "%navigation",
		"connect": do_navigation,
		"bind": true,
	},
	{
		"id": "%finalize",
		"connect": do_finalize,
		"bind": true,
	},
	{
		"id": "%save_single",
		"connect": save_single,
		"bind": true,
	},
	{
		"id": "%destroy_single",
		"connect": destroy_single,
		"bind": true,
	},
	{
		"id": "%bake_all",
		"connect": bake_all,
	},
	{
		"id": "%save_all",
		"connect": save_all,
	},
	{
		"id": "%destroy_all",
		"connect": destroy_all,
	},
	{
		"id": "%remove_all",
		"connect": remove_all,
	},
]

func _can_handle(object: Object) -> bool:
	if object is Node:
		var builder = fetch_builder_ancestor(object)
		if is_instance_valid(builder):
			return true
	return false

func _parse_begin(object: Object) -> void:
	panel = docker.instantiate()

	var builder: IslandBuilder = fetch_builder_ancestor(object)
	update_shapecount(builder)
	update_volume(builder)
	update_mass(builder)
	update_hitpoints(builder)

	var this_is_previous: bool = last_builder == builder
	if not this_is_previous:
		transforms.clear()

		if is_instance_valid(last_builder) and last_builder.is_inside_tree():
			unbind_realtime(last_builder)
			if last_builder.get_tree().process_frame.is_connected(_check_transforms):
				last_builder.get_tree().process_frame.disconnect(_check_transforms)

		bind_realtime(builder)

		if not builder.get_tree().process_frame.is_connected(_check_transforms):
			builder.get_tree().process_frame.connect(_check_transforms)

	last_builder = builder
	for btn in buttons:
		var button: Button = panel.get_node_or_null(btn["id"])
		if not is_instance_valid(button):
			push_warning("No button found for ID:", btn["id"])
			continue

		if btn.get("bind", false):
			button.pressed.connect(btn["connect"].bind(builder))
		else:
			button.pressed.connect(btn["connect"])

	var trealtime: CheckBox = panel.get_node("%toggle_realtime")
	trealtime.button_pressed = builder.realtime_preview
	trealtime.toggled.connect(_realtime_toggled)

	var tcsglint: CheckBox = panel.get_node("%toggle_csg_linter")
	tcsglint.button_pressed = csg_linting
	tcsglint.toggled.connect(_csg_linter)

	var tallowdestructive: CheckBox = panel.get_node("%toggle_allow_destructive")
	tallowdestructive.button_pressed = allow_destructive
	tallowdestructive.toggled.connect(toggle_allow_destructive)

	if csg_linting:
		lint_node(object)

	if not EditorInterface.get_inspector().property_edited.is_connected(on_property_change):
		EditorInterface.get_inspector().property_edited.connect(on_property_change)

	add_custom_control(panel)

func fetch_builder_ancestor(object: Node) -> IslandBuilder:
	if object is IslandBuilder:
		return object
	if not is_instance_valid(object.get_parent()) or object.get_tree().current_scene == object:
		return null
	return fetch_builder_ancestor(object.get_parent())

func update_shapecount(builder: IslandBuilder):
	panel.get_node("%shape_count").text = "{0} shapes".format([builder.get_shape_count()])
func update_volume(builder: IslandBuilder):
	panel.get_node("%volume").text = "%.2f m³" % builder.get_volume()
func update_mass(builder: IslandBuilder):
	var settings := builder.fetch_settings()
	panel.get_node("%mass").text = "%.2f kg" % (builder.get_volume() * settings.physics_density)
func update_hitpoints(builder: IslandBuilder):
	var settings := builder.fetch_settings()
	panel.get_node("%hitpoints").text = "%.2f HP" % (builder.get_volume() * settings.physics_health_density)

func do_serialize(builder: IslandBuilder):
	builder.serialize()
	update_shapecount(builder)
func do_mesh_preview(builder: IslandBuilder):
	builder.apply_mesh(builder.generate_preview_mesh(builder.target_mesh().mesh))
	update_volume(builder)
	update_mass(builder)
	update_hitpoints(builder)
func do_mesh_bake(builder: IslandBuilder):
	builder.apply_mesh(builder.generate_baked_mesh())
func do_collision(builder: IslandBuilder):
	var hulls = builder.generate_collision_hulls()
	builder.apply_collision_hulls(hulls, builder.get_volume())
func do_navigation(builder: IslandBuilder):
	builder.apply_navigation_properties(builder.generate_navigation_properties())
func do_finalize(builder: IslandBuilder):
	builder.realtime_preview = false
	panel.get_node("%toggle_realtime").button_pressed = false

	var t1 = Time.get_ticks_usec()
	builder.build(false)
	var t2 = Time.get_ticks_usec()
	print("IslandBuilder: FINALIZE took ", float(t2 - t1) * 0.001, " ms")

	# Hide builder if not target
	if builder.target() != builder:
		builder.visible = false

func save_single(builder: IslandBuilder):
	save_island(builder, true)
func save_all():
	if is_instance_valid(last_builder):
		for b in IslandBuilder.all_builders(last_builder.get_tree()):
			save_island(b, false)
		EditorInterface.get_resource_filesystem().scan()
func destroy_single(node: IslandBuilder):
	node.destroy_bakes()
	update_shapecount(node)
func destroy_all():
	if allow_destructive and is_instance_valid(last_builder):
		IslandBuilder.all_destroy_bakes(last_builder.get_tree())
		update_shapecount(last_builder)
	toggle_allow_destructive(false)
func bake_all():
	if is_instance_valid(last_builder):
		print("IslandBuilder: Building all islands...")
		IslandBuilder.all_bake(last_builder.get_tree())
		print("\t...all done!")

		# Wait for build data to be applied before updating statistics
		await last_builder.applied_build_data
		update_shapecount(last_builder)
		update_volume(last_builder)
		update_mass(last_builder)
		update_hitpoints(last_builder)
func remove_all():
	if allow_destructive and is_instance_valid(last_builder):
		var builders := IslandBuilder.all_builders(last_builder.get_tree())
		for builder in builders:
			builder.get_parent().remove_child(builder)
			builder.queue_free()
	toggle_allow_destructive(false)

func save_island(builder: IslandBuilder, scan: bool = true):
	if not is_instance_valid(builder):
		return

	var meta: Dictionary = {
		"scene": builder.get_tree().edited_scene_root.scene_file_path.get_basename(),
		"parent": builder.get_parent().name,
		"node": builder.name,
		"target": builder.target().name,
	}

	var filename := builder.fetch_settings().save_path.format(meta).simplify_path()
	if filename.is_empty():
		push_warning("IslandBuilder: No filename specified for the island, skipping...")
		return

	var dir := DirAccess.open("res://")
	var err := dir.make_dir_recursive(filename.get_base_dir())
	if err != OK:
		push_error("IslandBuilder: while creating filepath '{0}', got error {1}: {2}".format([
			filename, err, error_string(err)
		]))
		return
	dir = null # Close directory access

	if filename.ends_with(".scn") or filename.ends_with(".tscn"):
		var scene := PackedScene.new()
		var target := builder.target()
		# Change ownership of all children
		for child in target.get_children():
			if child is CollisionShape3D or child is MeshInstance3D:
				child.owner = target
		err = scene.pack(target)
		if err != OK:
			push_error("IslandBuilder: while packing '{0}', got error {1}: {2}".format([
				filename, err, error_string(err)
			]))
			return

		var flags := ResourceSaver.FLAG_CHANGE_PATH | ResourceSaver.FLAG_REPLACE_SUBRESOURCE_PATHS
		if filename.ends_with(".scn"):
			flags = flags | ResourceSaver.FLAG_COMPRESS
		err = ResourceSaver.save(scene, filename, flags)
		if err != OK:
			push_error("IslandBuilder: while saving '{0}', got error {1}: {2}".format([
				filename, err, error_string(err)
			]))
			return
		print("IslandBuilder: Saved '{0}' successfully".format([filename]))

		target.scene_file_path = filename
		target.set_editable_instance(target, true)
	elif filename.ends_with(".res") or filename.ends_with(".tres"):
		var mesh := builder.target_mesh().mesh

		var flags := ResourceSaver.FLAG_CHANGE_PATH | ResourceSaver.FLAG_REPLACE_SUBRESOURCE_PATHS
		if filename.ends_with(".res"):
			flags = flags | ResourceSaver.FLAG_COMPRESS

		err = ResourceSaver.save(mesh, filename, flags)
		if err != OK:
			push_error("IslandBuilder: while saving mesh '{0}', got error {1}: {2}".format([
				filename, err, error_string(err)
			]))
			return
		print("IslandBuilder: Saved mesh '{0}' successfully".format([filename]))
	else:
		push_warning("IslandBuilder: unknown file format while saving '{0}'".format([filename]))
		return

	if scan:
		EditorInterface.get_resource_filesystem().scan()

## Fetches all IslandBuilder nodes under the given node, inclusive
func fetch_all_builders(current: Node, builders: Array[IslandBuilder] = []) -> Array[IslandBuilder]:
	if current is IslandBuilder:
		builders.append(current)
	for child in current.get_children():
		fetch_all_builders(child, builders)
	return builders

func toggle_allow_destructive(new_val: bool) -> void:
	if not new_val and not allow_destructive: # don't recurse
		return

	allow_destructive = new_val
	if is_instance_valid(panel):
		if not new_val:
			panel.get_node("%toggle_allow_destructive").button_pressed = false
		panel.get_node("%destroy_all").disabled = not allow_destructive
		panel.get_node("%remove_all").disabled = not allow_destructive

## CSG LINTING ##

const LINT_PREFIXES = [
	"UNION_",
	"INTERSECT_",
	"SUBTRACT_",
]

func _csg_linter(new_state: bool):
	csg_linting = new_state
	if csg_linting and is_instance_valid(last_builder):
		lint_node_recursive(last_builder)

## Lints the given string with the according suffix
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

## Returns a material to represent the given CSG operation
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

## Performs CSG linting on the given node for better readability
func lint_node(node: Node):
	if is_instance_valid(last_builder) and last_builder.is_ancestor_of(node):
		if node is CSGShape3D:
			node.material_override = lint_material(node.operation)

		match node.get_class():
			"CSGBox3D":
				node.name = lint_name(node.name, node.operation, "box")
			"CSGSphere3D":
				node.name = lint_name(node.name, node.operation, "sphere")
			"CSGCylinder3D":
				node.name = lint_name(node.name, node.operation, "cylinder")
			"CSGTorus3D":
				node.name = lint_name(node.name, node.operation, "torus")

func lint_node_recursive(node: Node):
	for child in node.get_children():
		lint_node_recursive(child)
	lint_node(node)


## REALTIME PREVIEW ##

var realtime_queued: bool = false
var realtime_last_update: int = -1
var realtime_dirty: bool = false

## Initializes the realtime thread
func thread_init() -> void:
	EditorInterface.get_inspector().property_edited.connect(on_property_change)
## Deinitializes the realtime thread
func thread_deinit() -> void:
	pass

func _realtime_toggled(new_state: bool) -> void:
	if is_instance_valid(last_builder):
		last_builder.realtime_preview = new_state

## Unbind tree and property updates from mesh generation
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
## Bind tree and property updates to mesh regeneration
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
	update_realtime_preview()

func on_child_added(new_child: Node):
	bind_realtime(new_child, false)
	if csg_linting:
		lint_node(new_child)
func on_child_removed(new_child: Node):
	unbind_realtime(new_child)

func _check_transforms() -> void:
	if is_instance_valid(last_builder) and last_builder.realtime_preview:
		_check_transforms_internal(last_builder)

		var t = Time.get_ticks_msec()
		# If we have new changes, but haven't updated our generation in a while, do a clean pass to ensure we're at final
		if realtime_dirty and not realtime_queued:
			if t > realtime_last_update + TWEAK_TIMER_THRESHOLD:
				update_realtime_preview()
		# If somehow our thread failed, reset our queue status
		if realtime_queued and t > realtime_last_update + TWEAK_TIMEOUT_THRESHOLD:
			realtime_queued = false
func _check_transforms_internal(node: Node) -> void:
	if node is Node3D:
		var old_transform: Transform3D = transforms.get(node.get_instance_id(), node.transform)
		if not old_transform.is_equal_approx(node.transform):
			update_realtime_preview()
		transforms[node.get_instance_id()] = node.transform

	for child in node.get_children():
		_check_transforms_internal(child)

## Called if the IslandBuilder tree changed somehow
func update_realtime_preview():
	if is_instance_valid(last_builder) and last_builder.realtime_preview:
		last_builder.update_preview()
		realtime_last_update = Time.get_ticks_msec()
