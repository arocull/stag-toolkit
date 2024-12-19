extends Node

func _ready():
	# End test after frame
	StagTest.teardown.call_deferred()

	# Factorial tests
	StagTest.assert_true(is_nan(StagUtils.factorial(-1)), "Negative numbers are undefined")
	StagTest.assert_equal(1, StagUtils.factorial(0), "0! = 1")
	StagTest.assert_equal(1, StagUtils.factorial(1), "1! = 1")
	StagTest.assert_equal(6, StagUtils.factorial(3), "3! = 3 * 2 * 1 = 6")
	StagTest.assert_equal(120, StagUtils.factorial(5), "5! = 5 * 4 * 3 * 2 * 1 = 20 * 6 = 120")
	StagTest.assert_equal(620448401733239409999872.0, StagUtils.factorial(24), "24! = very large number")
	StagTest.assert_equal(2024, StagUtils.factorial(24) / (StagUtils.factorial(3) * StagUtils.factorial(24-3)), "nCr(24, 3)")
