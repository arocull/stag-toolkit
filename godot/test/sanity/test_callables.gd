extends Node

func _ready():
	StagTest.teardown.call_deferred()

#	print("Test Instantiation ---")
#	var sanity = StagSanityObject.new()
#	StagTest.assert_valid(sanity, "object should exist")
#
#	print("Test Instance Method Callable ---")
#	StagTest.assert_equal("1", sanity.stringify_int(1), "should stringify int")
#	StagTest.assert_true(sanity.stringify_int.is_valid(), "stringify_int should be valid callable")
#	StagTest.assert_equal("1", sanity.stringify_int.call(1), "should stringify int via callable")
#	StagTest.assert_equal("1", sanity.stringify_int.bind(1).call(), "should stringify int via binded callable")
#
#	print("Test Static Method Callable via Rust ---")
#	StagTest.assert_equal(3, StagSanityObject.return_int_via_callable_call(3), "Rust callable result should match input")
#	StagTest.assert_equal(3, StagSanityObject.return_int_via_callable_bind(3), "Rust callable result should match binded input")
#
#	print("Test Static Method Callable ---")
#	StagTest.assert_equal(1, StagSanityObject.return_int(1), "should return")
#	# The next 3 assertions cause script errors
#	StagTest.assert_true(StagSanityObject.return_int.is_valid(), "return_int should be valid callable")
#	StagTest.assert_equal(1, StagSanityObject.return_int.call(1), "should return int via callable")
#	StagTest.assert_equal(1, StagSanityObject.return_int.bind(1).call(), "should return int via binded callable")
#
#	print("Made it to end of test!") # Never prints!
