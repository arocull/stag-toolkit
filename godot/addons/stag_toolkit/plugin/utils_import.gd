extends RefCounted
class_name StagImportUtils
## Utility functions for use in scene importers.
## @experimental

## Recursively sets scene ownership of the given node.
static func fix_owner(node: Node, new_owner: Node):
	for child in node.get_children():
		fix_owner(child, new_owner)
	if not node == new_owner:
		node.owner = new_owner

## Generates a single convex hull for the mesh.
## Removes any PhysicsBodies that are children of the mesh, and instead makes the corresponding colliders
## children of the given parent.
## Returns an array of all collision objects found under the mesh, if generated.
static func generate_convex_hull(mesh: MeshInstance3D, parent: Node, simplify: bool = false) -> Array[CollisionShape3D]:
	var hulls: Array[CollisionShape3D] = []

	mesh.create_convex_collision(true, simplify)
	var cnt: int = 0
	for body in mesh.get_children():
		# Remove generated static or rigid bodies
		if body is StaticBody3D or body is RigidBody3D:
			# Migrate collision siblings up the tree
			for sibling in body.get_children():
				sibling.owner = null
				body.remove_child(sibling)
				if sibling is CollisionShape3D:
					hulls.append(sibling)
					sibling.name = "collis_{0}_{1}".format([mesh.name, cnt])
				cnt += 1
				parent.add_child(sibling)
				fix_owner(sibling, parent)
		# Remove body collision
		body.owner = null
		mesh.remove_child(body)
	return hulls

## Attempts to find an LOD suffix in the given string, returning -1 if not found.
static func get_lod_suffix(input_string: String) -> int:
	var suffix = input_string.substr(input_string.rfind("LOD"))
	if suffix.begins_with("LOD"):
		return suffix.substr(3).to_int()
	return -1  # Return -1 if the suffix does not match the expected pattern

## Recursively iterates over the given node and its children, returning a list of all MeshInstance3D nodes.
static func fetch_meshes(obj: Node, meshes: Array[MeshInstance3D] = []) -> Array[MeshInstance3D]:
	for child in obj.get_children():
		if child is MeshInstance3D:
			meshes.append(child)
		fetch_meshes(child, meshes)
	return meshes
