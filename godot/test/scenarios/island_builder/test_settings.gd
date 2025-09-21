extends Node

@export var settings: IslandBuilderSettings

func _ready() -> void:
	StagTest.teardown.call_deferred()

	var sig_changed := StagTest.signal_expector(settings.changed, "changed event")
	var sig_voxels := StagTest.signal_expector(settings.setting_changed_voxels, "voxels event")
	var sig_mesh := StagTest.signal_expector(settings.setting_changed_mesh, "mesh event")
	var sig_collision := StagTest.signal_expector(settings.setting_changed_collision, "collision event")

	print("Change material")
	settings.material_baked = StandardMaterial3D.new()
	sig_changed.assert_count(1)
	sig_changed.reset()

	var settings_voxels := settings.voxels
	var voxels_changed := StagTest.signal_expector(settings_voxels.setting_changed, "voxels changed")

	print("change SDF settings")
	settings.voxels.sdf_edge_radius += 0.5
	voxels_changed.assert_count(1)
	sig_changed.assert_count(1)
	sig_voxels.assert_count(1)
	sig_mesh.assert_count(0)
	sig_collision.assert_count(0)

	print("remove SDF settings")
	settings.voxels = null # Changing resource entirely changes things
	voxels_changed.assert_count(1)
	sig_changed.assert_count(2)
	sig_voxels.assert_count(2)
	sig_mesh.assert_count(0)
	sig_collision.assert_count(0)

	print("update SDF settings while not tracked")
	# Changing the voxel settings while unlinked does not affect the original settings resource
	settings_voxels.sdf_smooth_weight += 0.01
	voxels_changed.assert_count(2)
	sig_changed.assert_count(2)
	sig_voxels.assert_count(2)

	print("end of test")
