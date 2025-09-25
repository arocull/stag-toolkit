@tool
extends Node
class_name PrototypeAnimationSoup

class Pose extends RefCounted:
	var blends: Dictionary
	var positions: Dictionary
	var rotations: Dictionary
	var scales: Dictionary

@export var active: bool = true:
	set(newVal):
		active = newVal
		set_process(newVal)
@export var anim_lib: AnimationLibrary
@export var rig_path: NodePath

@export_range(0.0,2.0,0.1) var quick_time: float:
	set(newVal):
		quick_time = newVal
		sample_time = newVal
		#build_hashmap()
		#update_pose()
@export_range(-1.0,1.0,0.01,"or_greater") var blend_alpha: float = 0.5
@export_range(-1.0,1.0,0.01,"or_greater") var blend_speed: float = 1.0
@onready var sample_time: float = 0.0

@onready var anim1: Animation
@onready var anim2: Animation

@onready var rig: Node3D = $rig
@onready var skeleton: Skeleton3D

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	anim1 = anim_lib.get_animation("anim_run1")
	anim2 = anim_lib.get_animation("anim_run1_right")
	skeleton = $rig/armature_clover/Skeleton3D
	skeleton.reset_bone_poses()
	build_hashmap()

	print(anim1)
	print(anim2)

	if not active:
		set_process(false)

func get_node_path(base: NodePath) -> NodePath:
	return NodePath(StringName(rig_path) + "/" + base.get_concatenated_names())

@onready var hashmap: Dictionary
func build_hashmap():
	hashmap = {}

	for animation_name in anim_lib.get_animation_list():
		var anim = anim_lib.get_animation(animation_name)
		for i in anim.get_track_count():
			if hashmap.has(anim.track_get_path(i)): # Ignore track paths if we already have 'em
				continue

			var t = anim.track_get_type(i)

			var node = get_node_or_null(get_node_path(anim.track_get_path(i)))
			if is_instance_valid(node):
				if node is Skeleton3D:
					var bone_idx = skeleton.find_bone(anim.track_get_path(i).get_subname(0))
					hashmap[anim.track_get_path(i).hash()] = [node, bone_idx]
				elif node is MeshInstance3D and anim.track_get_type(i) == Animation.TYPE_BLEND_SHAPE:
					var shape_idx: int = node.find_blend_shape_by_name(anim.track_get_path(i).get_subname(0))
					hashmap[anim.track_get_path(i).hash()] = [node, shape_idx]

func blend_dictionary(a: Dictionary, b: Dictionary, alpha: float) -> void:
	for key in a.keys():
		if b.has(key):
			a[key] = lerp(a[key], b[key], alpha)
	a.merge(b, false)

func blend_poses(a: Pose, b: Pose, alpha: float) -> Pose:
	blend_dictionary(a.blends, b.blends, alpha)
	blend_dictionary(a.positions, b.positions, alpha)
	blend_dictionary(a.scales, b.scales, alpha)
	blend_dictionary(a.rotations, b.rotations, alpha)
	return a

func sample_pose(animation: Animation, _time: float) -> Pose:
	var pose: Pose = Pose.new()
	for i in animation.get_track_count():
		if animation.track_is_enabled(i):
			var ref = animation.track_get_path(i).hash()

			match animation.track_get_type(i):
				Animation.TYPE_POSITION_3D:
					pose.positions[ref] = animation.position_track_interpolate(i, sample_time)
				Animation.TYPE_ROTATION_3D:
					pose.rotations[ref] = animation.rotation_track_interpolate(i, sample_time)
				Animation.TYPE_SCALE_3D:
					pose.scales[ref] = animation.scale_track_interpolate(i, sample_time)
				Animation.TYPE_BLEND_SHAPE:
					pose.blends[ref] = animation.blend_shape_track_interpolate(i, sample_time)
	return pose

func apply_pose(pose: Pose):
	for key in pose.rotations.keys():
		if hashmap.has(key):
			var data: Array = hashmap.get(key)
			data[0].set_bone_pose_rotation(data[1], pose.rotations[key])
	for key in pose.positions.keys():
		if hashmap.has(key):
			var data: Array = hashmap.get(key)
			data[0].set_bone_pose_position(data[1], pose.positions[key])
	for key in pose.scales.keys():
		if hashmap.has(key):
			var data: Array = hashmap.get(key)
			data[0].set_bone_pose_scale(data[1], pose.scales[key])
	for key in pose.blends.keys():
		if hashmap.has(key):
			var data: Array = hashmap.get(key)
			data[0].set_blend_shape_value(data[1], pose.blends[key])

func update_pose(time: float):
	#skeleton.reset_bone_poses()

	var poseA = sample_pose(anim1, time)
	var poseB = sample_pose(anim2, time)
	var blended = blend_poses(poseA, poseB, blend_alpha)
	apply_pose(blended)

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	sample_time = wrapf(sample_time + delta * blend_speed, 0, anim1.length)
	update_pose(sample_time)
	#apply_pose(sample_pose(anim2, sample_time))
