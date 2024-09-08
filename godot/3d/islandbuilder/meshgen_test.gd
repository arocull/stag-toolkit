@tool
extends Node3D

@onready var aabb_checker = $aabb_checker
@export var do_build_mesh: bool = false:
	set(newVal):
		build_mesh()

func _ready() -> void:
	pass

func build_mesh():
	var mesh: ArrayMesh = $IslandBuilder.get_uv_sphere()
	$MeshInstance3D.mesh = mesh


func _on_island_builder_completed_serialize():
	aabb_checker.visibility_aabb = $IslandBuilder.get_aabb()
