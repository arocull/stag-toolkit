extends Node3D

var loose_unbound: StagTest.SignalExpector
var taut_snap: StagTest.SignalExpector

func _ready() -> void:
	loose_unbound = StagTest.signal_expector($loose_left.rope_unbound, 1, "loose rope unbound")
	taut_snap = StagTest.signal_expector($taut_right.rope_snapped, 1, "taut rope snapped")

func _on_settle_timer_timeout() -> void:
	# Read tension
	print("Force on loose RopeBinding: ", $loose_left.get_tension_force().length())
	StagTest.assert_equal(0.0, $loose_left.get_tension_force().length(), "loose rope should have no tension")

	# 1 meter displacement * spring constant
	var expected_tension: float = 1.0 * $rope_taut.fetch_settings().simulation_spring_constant
	print("Force on taut RopeBinding: ", $taut_right.get_tension_force().length())
	StagTest.assert_in_delta(expected_tension, $taut_right.get_tension_force().length(), 1e-6, "tight rope should have tension")

	loose_unbound.assert_not_emitted("loose rope should still be connected")
	$loose_left.bind_to = null
	loose_unbound.assert_emitted("loose rope should no longer be connected")

	taut_snap.assert_not_emitted("taut rope should not have snapped yet")

	$taut_right.snap_enabled = true

	# Wait a few physics ticks
	await StagTest.tick_timer_physics_process(3)
	StagTest.assert_true(not is_instance_valid($taut_right.bind_to), "taut_right should not be attached to anything")
	taut_snap.assert_emitted("taut rope should have snapped")

func _on_test_timeout_timeout() -> void:
	StagTest.teardown()
