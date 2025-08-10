extends EditorNode3DGizmoPlugin

func _get_gizmo_name() -> String:
	return "IslandBuilderAABB"

func _has_gizmo(node) -> bool:
	return node is IslandBuilder

func _init() -> void:
	create_material("main", Color(0.9,1.0,0.0))

func _redraw(gizmo: EditorNode3DGizmo) -> void:
	gizmo.clear()

	var builder := gizmo.get_node_3d() as IslandBuilder
	var aabb := builder.get_aabb()

	var lines: PackedVector3Array = [
		aabb.get_endpoint(0),
		aabb.get_endpoint(1),
		aabb.get_endpoint(0),
		aabb.get_endpoint(2),
		aabb.get_endpoint(0),
		aabb.get_endpoint(4),

		aabb.get_endpoint(7),
		aabb.get_endpoint(3),
		aabb.get_endpoint(7),
		aabb.get_endpoint(5),
		aabb.get_endpoint(7),
		aabb.get_endpoint(6),

		aabb.get_endpoint(1),
		aabb.get_endpoint(3),
		aabb.get_endpoint(1),
		aabb.get_endpoint(5),
		aabb.get_endpoint(4),
		aabb.get_endpoint(5),
		aabb.get_endpoint(4),
		aabb.get_endpoint(6),

		aabb.get_endpoint(2),
		aabb.get_endpoint(3),
		aabb.get_endpoint(2),
		aabb.get_endpoint(6),
	]

	gizmo.add_lines(lines, get_material("main", gizmo), false)
