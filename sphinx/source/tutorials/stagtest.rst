StagTest
================

Initial Setup
-----------------------

I heavily recommend adding Godot Engine to your PATH, so it can be run from a shell like so: ``$ godot``.

To use StagTest, add the script ``res://addons/stag_toolkit/plugin/stagtest.gd`` as ``StagTest`` to your project's list of Auto-Loads.
The order should not matter, but I recommend loading it last.

To verify that StagTest is working, cd into your project directory, and run: ``$ godot --headless --stagtest?`` to get help output.

Testing Architecture
-------------------------------

StagTest utilizes Godot's `SceneTree`_ as the test runtime to allow for fully simulated gameplay.
This means that SceneTree-related node events like ``_init()``, ``_ready()``, ``_process()`` and even ``_enter_tree()`` and ``_exit_tree()`` will be called.

In order to operate inside the SceneTree, **tests must be contained within scenes**,
rather than run as scripts.
StagTest will automatically look for scene files inside ``res://test/scenarios`` to run as tests,
but you can manually point to a directory or specific scene with the ``--test=<TestPath>`` flag.

When running test scenes, they are loaded as a `PackedScene`_ with caching disabled,
before calling ``change_scene_to_packed(...)`` on the SceneTree.
This is when the test starts.

.. _SceneTree: https://docs.godotengine.org/en/stable/classes/class_scenetree.html
.. _PackedScene: https://docs.godotengine.org/en/stable/classes/class_packedscene.html

Test Teardown
^^^^^^^^^^^^^^^^^^^^^^^
**Each test is simulated indefeinitely until it indicates that it has completed,
or times until StagTest times out the test** (30 seconds by default).
You can do this by calling ``StagTest.teardown()`` to notify StagTest to teardown the test.

During the teardown process, the scene is removed from the tree before being unloaded.
This means events like ``_exit_tree()`` are still called, and upon error, will fail the test.

Startup Scene
^^^^^^^^^^^^^^^^^^^^^^^^^

Your startup scene will be loaded (and unloaded immediately afterward) while running StagTest.
To prevent your scene from running ``_ready()`` code, return if testing is active.

.. code-block:: gdscript
	:caption: startup_scene.gd

	func _ready():
		# DON'T switch scenes if StagTest is active
		if is_instance_valid(StagTest) and StagTest.is_active():
			return

It is necessary to check whether the singleton exists via ``is_instance_valid(StagTest)``, as StagTest will remove itself from the tree upon entering the ready state when running in a release build.

Custom Exit
^^^^^^^^^^^^^^^^^^^^^^^^^

If your game has a custom quit function, you can tell StagToolkit to use that instead of a traditional ``get_tree().quit()``.
The Callable must accept an exit code.

.. code-block:: gdscript
	:caption: custom_exit.gd

	func quit(exit_code: int = 0):
		quitting.emit()

		if Utils.is_debug():
			print("QUITTING, here's the orphans:")
			Node.print_orphan_nodes()

		get_tree().quit(exit_code)

	func _ready():
		# Force StagToolkit to use our own teardown sequence
		StagTest.override_exit_function(quit)


Testing
----------------

For basic unit tests, I usually create an empty scene with a single Node and a script attached, like so.

.. code-block:: gdscript

	extends Node
	func _init():
		print("I'm INITIALIZED!")
		StagTest.assert_true(true, "this is optional context")
	func _enter_tree():
		print("I'm ENTERING TREE!")
		StagTest.assert_equal(1, 1, "ensure two variants are equal")
	func _ready():
		print("I'm READY!")
		StagTest.assert_unequal(1, 2, "ensure two variants are NOT equal")
		StagTest.assert_valid(self, "ensure that the provided Object is valid")
	func _process(delta):
		print("I'm PROCESSING for {0} seconds!".format([delta]))
		StagTest.teardown()
	func _exit_tree():
		print("I'm even EXITING THE TREE!")

Because tests are composed of scenes,
you may choose to have any number of nodes in your scene orchestrating the test.

From a flat ground plane with a single character,
to using an entire level as your root node with various pen-testing nodes attached,
you can get as creative as you want with this!

Examples
^^^^^^^^^^^^

Example tests are located under `stag-toolkit/godot/test/scenarios/example <https://github.com/arocull/stag-toolkit/tree/master/godot/test/scenarios/example>`_.
For more complicated examples, see all tests in the parent directory.

API
^^^^^^^^^^^^

Refer to the :doc:`API reference <../plugin/class_stagtest>`,
or check the docs in-engine using the "Search Help" menu (F1 hotkey).

StagTest has its own customized assertions,
but regular Godot assertions (and custom assertions that push errors) also work.
