@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
extends EditorScenePostImportPlugin

func _get_import_options(_path: String):
	# If true, will generate LODs based on the suffix of the mesh name.
	# Example: `cactus_LOD0`, `cactus_LOD1` and `cactus_LOD2` will define:
	# LOD0 (high quality), LOD1 (medium quality), and LOD2 (low quality) for the same mesh respectively.
	#
	# **LODs utilize visibility ranges with multiple nodes**, rather than Godot's native LOD implementation,
	# as it does not allow for custom LODs.
	# This allows the systems to be used in combination.
	add_import_option_advanced(TYPE_BOOL, "stag_toolkit/simple_lod/custom_lods",
		false, PROPERTY_HINT_NONE)

	# Distance at which the lowest LOD will be used. All available LODs are evenly distributed.
	add_import_option_advanced(TYPE_FLOAT, "stag_toolkit/simple_lod/lod_distance",
		50.0, PROPERTY_HINT_RANGE, "0.0,1000.0,0.1,or_greater")

	# If true, will generate simplified collision hulls for ALL meshes in scene.
	# The collision hulls will NOT have corresponding Physics Bodies, so they can be reused.
	# If LODs are enabled, generates a collision hull using the lowest LOD.
	add_import_option_advanced(TYPE_BOOL, "stag_toolkit/simple_lod/collision_hulls",
		false, PROPERTY_HINT_NONE)

func _post_process(scene: Node):
	var generate_lods: bool = get_option_value("stag_toolkit/simple_lod/custom_lods")
	var generate_hulls: bool = get_option_value("stag_toolkit/simple_lod/collision_hulls")

	# List of LODs
	var lods: Dictionary = {}

	if generate_lods:
		# Stores the given node under the appropriate LOD
		var store_lod: Callable = func(obj: GeometryInstance3D, lod: int) -> void:
			var items: Array = lods.get(lod, [])
			if not lods.has(lod):
				lods[lod] = items
			items.append(obj)

		# Iterate through all children to fetch LODs
		fetch_lods(scene, store_lod)

		# Setup LODs
		var thresh: float = float(get_option_value("stag_toolkit/simple_lod/lod_distance") / float(max(lods.size() - 1, 1)))
		for lod in lods.keys():
			var items: Array = lods[lod]

			for child: GeometryInstance3D in items:
				child.add_to_group("lod{0}".format([lod]), true)
				child.visibility_range_begin = thresh * lod

				if not lod == (lods.size() - 1): # Only set end of visiblity range for non-final LODs
					child.visibility_range_end = thresh * (lod + 1)

	# Generate collision hulls
	if generate_hulls:
		var meshes: Array[MeshInstance3D] = StagImportUtils.fetch_meshes(scene)

		# See if we manually specify collision
		var collision_meshes: Array[MeshInstance3D] = []
		for mesh in meshes:
			# If we do find meshes with collision specified, add them to the list
			if mesh.name.to_lower().begins_with("collis_") or mesh.name.to_lower().begins_with("collision_"):
				collision_meshes.append(mesh)

		if collision_meshes.size() > 0:
			for mesh in collision_meshes:
				StagImportUtils.generate_convex_hull(mesh, scene)
				mesh.queue_free()
		else:
			for mesh in meshes:
				# If the mesh has an LOD, only generate convex hulls for lowest LODs
				var lod: int = StagImportUtils.get_lod_suffix(mesh.name)
				if not generate_lods or (lod < 0 or lod == (lods.size() - 1)):
					StagImportUtils.generate_convex_hull(mesh, scene)

func fetch_lods(obj: Node, store_lod: Callable):
	for child in obj.get_children():
		if child is GeometryInstance3D:
			var lod = StagImportUtils.get_lod_suffix(child.name)
			if lod >= 0:
				store_lod.call(child, lod)
		fetch_lods(child, store_lod)
