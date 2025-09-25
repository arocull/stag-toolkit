extends Node

func _ready() -> void:
	# Create a new logger
	var logger := StagLogger.new()
	# Customize logger as desired...
	logger.log_level_buffer = StagLogger.LogLevel.LLInfo
	logger.log_level_console = StagLogger.LogLevel.LLDebug
	logger.flush_interval = 3
	# Register logger with Engine
	OS.add_logger(logger)

	var fileError := logger.create_buffer()
	print("file error: ", error_string(fileError))
	StagTest.assert_equal(OK, fileError, "logger should create temporary file")
	StagTest.assert_valid(logger.buffer, "logger should have a buffer now")
	StagTest.assert_equal(logger._messages_until_flush, 3, "flush counter should be set")

	# Do some logging....
	logger.debug("debug message")
	StagTest.assert_equal(logger._messages_until_flush, 3, "flush counter does not decrement with ignored log")
	logger.info("info message")
	StagTest.assert_equal(logger._messages_until_flush, 2, "flush counter should be going down")
	logger.warn("warning message")
	logger.error("error message")

	# Logger should have flushed
	StagTest.assert_equal(logger._messages_until_flush, 3, "should have flushed automatically")

	# Logfile contents should be what we expect
	StagTest.assert_equal(
		"info message\nwarning message\nerror message\n",
		logger.buffer.get_as_text(),
		"should ignore debug log level",
	)

	# Save and close log file
	logger.close()

	StagTest.assert_equal(null, logger.buffer, "buffer should no longer be set")
	OS.remove_logger(logger)

	StagTest.teardown()
