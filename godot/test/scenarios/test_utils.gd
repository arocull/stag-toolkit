extends Node

func _ready():
	# End test after frame
	StagTest.teardown.call_deferred()

	# Factorial tests
	print("Factorial")
	StagTest.assert_true(is_nan(StagUtils.factorial(-1)), "Negative numbers are undefined")
	StagTest.assert_equal(1, StagUtils.factorial(0), "0! = 1")
	StagTest.assert_equal(1, StagUtils.factorial(1), "1! = 1")
	StagTest.assert_equal(6, StagUtils.factorial(3), "3! = 3 * 2 * 1 = 6")
	StagTest.assert_equal(120, StagUtils.factorial(5), "5! = 5 * 4 * 3 * 2 * 1 = 20 * 6 = 120")
	StagTest.assert_equal(620448401733239409999872.0, StagUtils.factorial(24), "24! = very large number")
	StagTest.assert_equal(2024, StagUtils.factorial(24) / (StagUtils.factorial(3) * StagUtils.factorial(24-3)), "nCr(24, 3)")

	print("Defaults")
	var d: Dictionary = {
		"test": "1.0",
		"coolness": true,
		Vector2.ZERO: 5,
	}

	StagTest.assert_equal(1.0, StagUtils.default(d, "test", TYPE_FLOAT), "invalid types should be converted when possible")
	StagTest.assert_equal(0.0, StagUtils.default(d, "test", TYPE_FLOAT, 0.0), "invalid types should use override when provided")
	StagTest.assert_equal(true, StagUtils.default(d, "coolness", TYPE_BOOL), "valid types retain their value")
	StagTest.assert_equal(5, StagUtils.default(d, Vector2.ZERO, TYPE_INT), "non-string keys can be used")
	StagTest.assert_equal(false, StagUtils.default(d, "deer", TYPE_BOOL), "non-existent keys use the corresponding type fallback")
	StagTest.assert_equal(true, StagUtils.default(d, "deer", TYPE_BOOL, true), "non-existent keys use overrides when provided")
	StagTest.assert_equal("", StagUtils.default(d, "deer", TYPE_STRING), "non-existent keys keep empty strings")
	StagTest.assert_true(typeof(&"") == typeof(StagUtils.default(d, "deer", TYPE_STRING_NAME)), "should retain proper type when loading default")
