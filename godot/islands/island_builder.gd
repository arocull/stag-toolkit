@tool
extends Node

@onready var island_builder: IslandBuilder = %IslandBuilder

@export var serialize: bool:
	set(newVal):
		serialize = false
		if newVal:
			_serialize()
@export var build: bool:
	set(newVal):
		serialize = false
		if newVal:
			_generate()

@export var test: bool:
	set(newVal):
		test = false
		if newVal:
			_test()

enum BuilderShape {
	Box = 0,
	Sphere = 1,
}

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if not Engine.is_editor_hint():
		_serialize()
		_generate()

func _generate():
	var mesh: ArrayMesh = island_builder.generate_mesh()
	$world/mesh_preview.mesh = mesh
	$world/mesh_preview.visible = true

func _serialize():
	island_builder.serialize()
	$world/aabb_preview.visibility_aabb = island_builder.get_aabb_padded()

func _serialize_walk(node: Node):
	for child in node.get_children():
		_serialize_walk(child)
	
	if (node is CSGBox3D or node is CSGSphere3D) and node.visible:
		var t: Transform3D = island_builder.global_transform.affine_inverse() * node.global_transform
		var shape: IslandBuilderShape = IslandBuilderShape.new()
		shape.position = t.origin
		#shape.rotation = t.basis.get_euler()
		shape.rotation = t.basis.get_rotation_quaternion().get_euler(EULER_ORDER_ZXY)  # Ensure pitch, yaw, roll are on expected axii (XYZ)
		#print(rad_to_deg(shape.rotation.x), ",\t ", rad_to_deg(shape.rotation.y), ",\t ", rad_to_deg(shape.rotation.z))
		shape.scale = t.basis.get_scale()
		
		if node is CSGBox3D:
			shape.scale *= node.size
		if node is CSGSphere3D:
			shape.shape = BuilderShape.Sphere
			shape.radius = node.radius
		
		island_builder.shapes.append(shape)

const TEST_INPUTS: PackedVector3Array = [Vector3.ZERO, Vector3.UP * 0.25, Vector3.UP * 0.5, Vector3.UP * 0.6, Vector3.UP, Vector3.UP * 2.0]
const TEST_OUPUTS: PackedFloat32Array = [1.0, 0.5]
const EPSILON = 1e-5

func _test():
	print("\n---- STARTING TEST ----")
	var shape = island_builder.shapes[0]
	#var shape = IslandBuilderShape.new()
	#shape.shape = BuilderShape.Box
	#shape.radius = 0.0
	#shape.scale = Vector3.ONE * 2.0
	#shape.position = Vector3.ZERO
	#shape.edge_radius = 0.0
	
	for i in range(0,TEST_INPUTS.size()):
		var d: float = shape.distance(TEST_INPUTS[i])
		var d2: float = _test_formula(shape, shape.to_local(TEST_INPUTS[i]))
		print("Case ", i, ": ", d, "\tversus: ", d2)
		#assert(absf(d - TEST_OUPUTS[i]) < EPSILON, "Case {0}: With input {1}\t, expected {2},\t but got {3}".format([i, TEST_INPUTS[i], TEST_OUPUTS[i], d]))

func _test_formula(shape: IslandBuilderShape, local_pos: Vector3) -> float:
	var q = local_pos.abs() - (shape.scale / 2) + Vector3(shape.edge_radius, shape.edge_radius, shape.edge_radius)
	var m = Vector3(max(q.x, 0), max(q.y, 0), max(q.z, 0))
	return m.length() + min(q[q.max_axis_index()], 0.0) - shape.edge_radius
