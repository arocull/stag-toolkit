extends Node

func _ready():
	StagTest.teardown.call_deferred()

	StagTest.benchmark(StagTest.assert_equal.bind(1, 1, "assertion test"), 1000, "StagTest.assert_equal")
