extends Node

func _ready():
	# Teardown test after frame
	StagTest.teardown.call_deferred()

	# Create and allocate a float queue
	var queue: QueueFloat = QueueFloat.new()
	queue.allocate(5)

	# Assert that we properly resized it
	StagTest.assert_equal(5, queue.size(), "queue length")
	StagTest.assert_equal(0, queue.index(), "queue index")

	# Push elements to the queue
	queue.push(9.0)
	queue.push(-3.0)
	queue.push(2.0)
	queue.push(-1.5)
	queue.push(17.0)

	# Assert sorting
	var arr: PackedFloat32Array = [-3.0, -1.5, 2.0, 9.0, 17.0]
	StagTest.assert_equal(arr, queue.sorted(), "sorted")

	# Assert statistics
	StagTest.assert_approx_equal(4.7, queue.mean(), 1e-9, "mean")
	StagTest.assert_approx_equal(2.0, queue.median(), 1e-9, "median")
	StagTest.assert_true(Vector2(-3.0, 17.0).is_equal_approx(queue.range()), "range should be equal")
	StagTest.assert_approx_equal(7.413501, queue.standard_deviation(), 1e-5, "standard deviation")
