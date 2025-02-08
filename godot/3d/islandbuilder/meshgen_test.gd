@tool
extends Node3D

@export var check_aabb: bool = false:
	set(new_val):
		if is_inside_tree():
			$aabb_checker.visibility_aabb = $IslandBuilder.get_aabb()

@export var navigation_properties: NavIslandProperties = null

func set_navigation_properties(props: NavIslandProperties):
	navigation_properties = props
