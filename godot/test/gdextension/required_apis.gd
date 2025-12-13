extends Node

@onready var island_builder: IslandBuilder = $Node3D/IslandBuilder
@onready var mesh_island: MeshInstance3D = $Node3D/IslandBuilder/mesh_island
@onready var mesh: ArrayMesh = mesh_island.mesh

func _ready() -> void:
	# Populat mesh data
	island_builder.serialize()
	island_builder.generate_preview_mesh(mesh)

	# Test ArrayMesh
	mesh_island.mesh.surface_set_material(0, null)
	print(mesh_island.mesh.surface_get_material(0))
	#mesh.clear_surfaces()
	print(mesh.create_placeholder())
	print(mesh.create_trimesh_shape())
	print(mesh.create_convex_shape())
	print(mesh.create_outline(0.1))
	print(mesh.generate_triangle_mesh())

	# Test PrimitiveMesh
	var primmesh := BoxMesh.new()
	primmesh.material = null
	print(primmesh.material)

	StagTest.teardown()
