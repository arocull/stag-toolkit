@tool
extends Node

@onready var island_builder: IslandBuilder = %IslandBuilder

@export var serialize: bool:
	set(newVal):
		serialize = false
		if Engine.is_editor_hint():
			_serialize()
			print("Serialized")

enum BuilderShape {
	Box = 0,
	Sphere = 1,
}

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	$world/IslandBuilder.generate()
	
	for shape: IslandBuilderShape in island_builder.shapes:
		print(shape.distance(Vector3.ZERO))
	
	if not Engine.is_editor_hint():
		_serialize()

func _serialize():
	island_builder.shapes.clear()
	_serialize_walk(island_builder)
	$world/aabb_preview.visibility_aabb = island_builder.get_aabb()

func _serialize_walk(node: Node):
	for child in node.get_children():
		_serialize_walk(child)
	
	if (node is CSGBox3D or node is CSGSphere3D) and node.visible:
		var t: Transform3D = island_builder.global_transform.inverse() * node.global_transform
		var shape: IslandBuilderShape = IslandBuilderShape.new()
		shape.position = t.origin
		shape.rotation = t.basis.get_euler()
		shape.rotation = t.basis.get_euler(2) # Get rotation in YXZ axis order
		shape.scale = t.basis.get_scale()
		
		if node is CSGBox3D:
			shape.scale *= node.size
		if node is CSGSphere3D:
			shape.shape = BuilderShape.Sphere
			shape.radius = node.radius
		
		print(shape.get_corners())
		print(shape.get_aabb())
		island_builder.shapes.append(shape)
