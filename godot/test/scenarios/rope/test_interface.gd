extends Node3D

func _ready():
	StagTest.teardown.call_deferred()

	StagTest.assert_equal(
		preload("res://assets/rope/simulated_rope_defaults.tres"),
		%rope_default.fetch_settings(),
		"rope without specified settings should match defaults"
	)

	StagTest.assert_equal(
		%rope_custom.settings,
		%rope_custom.fetch_settings(),
		"rope with specific settings should use said settings"
	)

	%rope_default.settings = %rope_custom.fetch_settings()
	StagTest.assert_equal(
		%rope_custom.settings,
		%rope_default.fetch_settings(),
		"fetch settings respects run-time overrides"
	)
