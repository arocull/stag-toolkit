@icon("res://addons/stag_toolkit/icons/icon_stagtoolkit_monochrome.svg")
class_name StagLogger
extends Logger
## Simple [Logger] wrapper that allows for script error detection.
##
## Allows logging with log levels, and can optionally include Engine logs and error detection.
##[br][br]
## The logger supports additional customization,
## such as console colors and log levels [member StagLogger.log_level_console].
##[br][br]
## If the logger is registered with the engine via [method OS.add_logger],
## pushed errors and warnings will automatically be written to the buffer (if one exists).
##[codeblock]
##var logger := StagLogger.new()
### Customize logger as desired...
##logger.log_level_console = StagLogger.LogLevel.LLDebug
##logger.log_level_buffer = StagLogger.LogLevel.LLInfo
##logger.color_warn = Color.YELLOW
##logger.flush_interval = 50
### ...
##
##logger.create_buffer(false) # Optionally create a temp file for writing to
##OS.add_logger(logger) # Optionally register logger with Engine for error catching
##
### Do some logging.
##logger.debug("debug message")
##logger.info("info message")
##logger.warn("warning message")
##logger.error("error message")
##
### Clean up the logger.
##logger.close()
##OS.remove_logger(logger) # Unregister logger from engine
##[/codeblock]

enum LogLevel {
	## Show all messages.
	LLDebug = 0,
	## Show informational messages, warnings, and errors.
	LLInfo = 1,
	## Only show warnings and errors.
	LLWarn = 2,
	## Only show errors.
	LLError = 3
}

## Emitted when Godot Engine passes a warning.
signal event_warning(message: String)
## Emitted when Godot Engine passes an error.
signal event_error(message: String)
## Emitted when Godot Engine passes a script error.
signal event_error_script(message: String)
## Emitted when Godot Engine passes a shader error.
signal event_error_shader(message: String)

## Console coloration for debug logs.
var color_debug: Color = Color.GRAY
## Console coloration for information logs.
var color_info: Color = Color.WHITE
## Console coloration for warning logs.
var color_warn: Color = Color.ORANGE
## Console coloration for error logs.
var color_error: Color = Color.RED

## Error string for automatic error logging (i.e. script or shader errors).
var error_string: String = "{file}@{line} (type {error_type}) -> {function}: {code}{rationale}{backtrace}"
## Whether to include backtraces in automatic error logs.
var error_backtraces: bool = true

## What logs are actually written to the buffer.
var log_level_buffer: LogLevel = LogLevel.LLDebug
## What logs are actually written to the console.
var log_level_console: LogLevel = LogLevel.LLDebug

## Temporary log file where buffer writes are written to.
## I would do this as a memory file, but it's [url=https://github.com/godotengine/godot/pull/98287]not yet implemented[/url].
var buffer: FileAccess

## How many messages can be passed before a flush occurs automatically.
##[br][br]
## Flushes write buffer data to the logfile, which can be expensive if done frequently.
## However, if data is not flushed, it may be missing from the file if the engine crashes.
##[br][br]
## In normal conditions, the file should be flushed automatically when dereferencing the logger,
## or if you call [method StagLogger.close].
var flush_interval: int = 50
## Immediately flush after writing an error, regardless of how many lines were written.
var flush_after_error: bool = true
var _messages_until_flush: int = 0

# Called from push_warning, push_error, and Engine errors.
# Errors are already printed to console, so just save these to the log and emit events.
func _log_error(function: String, file: String, line: int, code: String, rationale: String, editor_notify: bool, error_type: int, script_backtraces: Array[ScriptBacktrace]) -> void:
	var backtrace: String = ""
	if error_backtraces:
		backtrace = "\n\tBacktrace:"
		for backtrace_line in script_backtraces:
			backtrace += "\n"+backtrace_line.format(0, 4)

	var message := error_string.format({
		"function": function,
		"file": file,
		"line": line,
		"code": code,
		"rationale": rationale,
		"error_type": error_type,
		"backtrace": backtrace,
	})

	match error_type:
		Logger.ErrorType.ERROR_TYPE_WARNING:
			if log_level_buffer <= LogLevel.LLWarn:
				_write_buffer(message, false)
		_:
			if log_level_console <= LogLevel.LLError:
				_write_buffer(message, true)

	# Fire off corresponding events
	match error_type:
		Logger.ErrorType.ERROR_TYPE_WARNING:
			event_warning.emit(message)
		Logger.ErrorType.ERROR_TYPE_ERROR:
			event_error.emit(message)
		Logger.ErrorType.ERROR_TYPE_SCRIPT:
			event_error_script.emit(message)
		Logger.ErrorType.ERROR_TYPE_SHADER:
			event_error_shader.emit(message)

# Called from regular print functions.
func _log_message(message: String, is_error: bool) -> void:
	pass # Don't double up on logging!

## Logs a message with the "debug" log level.
func debug(message: String) -> void:
	if log_level_console <= LogLevel.LLDebug:
		_write_console(message, color_debug)
	if log_level_buffer <= LogLevel.LLDebug:
		_write_buffer(message)
## Logs a message with the "info" log level.
func info(message: String) -> void:
	if log_level_console <= LogLevel.LLInfo:
		_write_console(message, color_info)
	if log_level_buffer <= LogLevel.LLInfo:
		_write_buffer(message)
## Logs a message with the "warn" log level.
##[br][br]
## If [code]use_push[/code] is true, it will use [method @GlobalScope.push_warning] instead.
## This will be logged to the buffer with a stacktrace if the logger is registered with the Engine.
func warn(message: String, use_push: bool = false) -> void:
	if use_push:
		push_warning(message)
		return

	if log_level_console <= LogLevel.LLWarn:
		_write_console(message, color_warn)
	if log_level_buffer <= LogLevel.LLWarn:
		_write_buffer(message)
## Logs a message with the "error" log level.
##[br][br]
## If [code]use_push[/code] is true, it will use [method @GlobalScope.push_error] instead.
## This will be logged to the buffer with a stacktrace if the logger is registered with the Engine.
func error(message: String, use_push: bool = false) -> void:
	if use_push:
		push_error(message)
		return

	if log_level_console <= LogLevel.LLError:
		_write_console(message, color_error)
	if log_level_buffer <= LogLevel.LLError:
		_write_buffer(message, true)

func _write_console(message: String, color: Color) -> void:
	print_rich("[color={1}]{0}[/color]".format([message, color.to_html(false)]))

func _write_buffer(message: String, is_error: bool = false) -> void:
	if is_instance_valid(buffer):
		buffer.store_line(message)

		_messages_until_flush -= 1
		if _messages_until_flush <= 0 or (is_error and flush_after_error):
			flush()

## Initializes a temporary file for writing to. Closes the existing buffer if there is one.
## You can choose to provide your own file with [method StagLogger.set_buffer].
##[br][br]
##[code]keep[/code] sets whether cause the file persists in your temp directory after closing.
##[code]prefix[/code] sets the prefix of the file for organizational purposes.
##[br][br]
## Returns [code]OK[/code] if the buffer was successfully opened, or a filesystem [enum Error] otherwise.
func create_buffer(keep: bool = false, prefix: String = "log") -> Error:
	var newBuffer := FileAccess.create_temp(
		FileAccess.WRITE_READ,
		prefix, "log", keep,
	)
	if not is_instance_valid(newBuffer):
		return FileAccess.get_open_error()
	set_buffer(newBuffer)
	return OK

## Sets the file for writing to. Closes the existing buffer if there is one.
func set_buffer(new_buffer: FileAccess) -> void:
	if is_instance_valid(buffer):
		close()
	_messages_until_flush = flush_interval
	buffer = new_buffer

## Flushes the buffer, writing directly to the logfile.
func flush() -> void:
	_messages_until_flush = flush_interval
	if is_instance_valid(buffer):
		buffer.flush()

## Flushes and closes the logfile.
func close() -> void:
	if is_instance_valid(buffer):
		buffer.flush()
		buffer.close()
	buffer = null
