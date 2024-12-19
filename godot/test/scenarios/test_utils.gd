extends Node

func _ready():
	# End test after frame
	StagTest.teardown.call_deferred()

	# Factorial tests
	StagTest.assert_equal(0, StagUtils.factorial(-1), "Negative numbers are undefined")
	StagTest.assert_equal(1, StagUtils.factorial(0), "0! = 1")
	StagTest.assert_equal(1, StagUtils.factorial(1), "1! = 1")
	StagTest.assert_equal(6, StagUtils.factorial(3), "3! = 3 * 2 * 1 = 6")
	StagTest.assert_equal(120, StagUtils.factorial(5), "5! = 5 * 4 * 3 * 2 * 1 = 20 * 6 = 120")
