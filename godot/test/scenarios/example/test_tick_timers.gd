extends Node

var process_ticks_local: int = 0
var physics_ticks_local: int = 0

var process_ticks_stagtest: int = 0
var physics_ticks_stagtest: int = 0

func _ready() -> void:
	set_process(true)
	set_physics_process(false)

	# Bind to process tick events for analysis
	StagTest.tick_process.connect(process_incr)

	# Await 10 process tick notifications
	await StagTest.tick_timer_process(10)
	# Resumes coroutine at the very beginning of tick 10 before anything else processes

	StagTest.assert_equal(10, process_ticks_stagtest, "awaited 10 process ticks")
	StagTest.assert_equal(9, process_ticks_local, "nodes are just about to process tick 10")


	# Finish this tick, and wait 5 more ticks
	await StagTest.tick_timer_process(5)
	StagTest.assert_equal(15, process_ticks_stagtest, "awaited 15 process ticks")
	StagTest.assert_equal(14, process_ticks_local, "all other nodes finished 14 ticks")


	set_process(false)
	set_physics_process(true)
	StagTest.tick_physics_process.connect(physics_incr)

	# Await 10 physics process ticks
	await StagTest.tick_timer_physics_process(10)

	StagTest.assert_equal(10, physics_ticks_stagtest, "awaited 10 physics process ticks")
	StagTest.assert_equal(9, physics_ticks_local, "all nodes processed 10 physics ticks")


	# Finish this tick, and wait 5 more ticks
	await StagTest.tick_timer_physics_process(5)
	StagTest.assert_equal(14, physics_ticks_local, "awaiting an additional 5 physics ticks")


	# Can also await individual ticks
	await StagTest.tick_process
	await StagTest.tick_physics_process


	StagTest.teardown()

func _process(_delta: float) -> void:
	process_ticks_local += 1
func _physics_process(_delta: float) -> void:
	physics_ticks_local += 1

func process_incr():
	process_ticks_stagtest += 1
func physics_incr():
	physics_ticks_stagtest += 1
