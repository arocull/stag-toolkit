# SimulatedRope

A quick overview on how to use `SimulatedRope` nodes.

## Initial Setup

To use the rope, you need a tiny bit of project setup to tell Godot how to render the rope simulation.

1. [Install StagToolkit](../readme.md#installation)
2. Create a new `SimulatedRopeSettings` resource in your project.
3. Go to Project Settings > Addons > Stag Toolkit, and assign "Simulated Rope > Default Settings" to your created settings resource. This will provide a nice default for all rope simulations to use. (Setting path is `addons/stag_toolkit/simulated_rope/default_settings`)
4. In order to render rope simulations, you need a custom mesh and shader. Some example assets are provided in [godot/assets/rope](../godot/assets/rope/). I highly reccommend using these in your project to start.
5. Finally, assign the provided `Mesh` and `ShaderMaterial` to the "Render Mesh" and "Render Material" properties of the `SimulatedRopeSettings` accordingly.

## Making a Rope

Now that your project is ready for rope simulation, here is how you get started with it!

1. Create a 3D scene.
2. Add a `SimulatedRope` node to the scene.
3. Add two `SimulatedRopeBinding` nodes to the scene, and move them away from each other. If the bindings are left in the same position, the simulation may encounter errors.
4. On the second `SimulatedRopeBinding` node, set the "Bind At" property to `1.0` instead of `0.0`. This will tell the binding to hold onto the opposite end of the rope.
5. On both `SimulatedRopeBinding` nodes, set the "Bind To" property to the `SimulatedRope` node you spawned initially. This will attach the rope to the bindings.
6. At this point, you should see the rope (which probably fell through the world). Wait for the rope to settle, or reload the scene.
7. If desired, override the rope settings by setting the "Settings" property on the `SimulatedRope`, or change your defaults. Just remember to assign a "Render Mesh" and "Render Material" again!
    - Sometimes the rope might not update when changing settings, so you may have to reload your scene to see changes.

## Notes

- `SimulatedRope` and associated classes only work in 3D environments.
- If a `SimulatedRopeBinding` is a child of a `RigidBody3D`, the binding will apply tension force at its position relative to the `RigidBody3D`.
- If you have a custom physics implementation or are using a `CharacterBody3D`, see `SimulatedRope.get_tension_force_at`.
- It is highly recommended to multithread the rope simulation step if you have lots of ropes in your scene. For details, read the documentation on the `simulation_tick_on_physics` property for `SimulatedRopeSettings`.
    - In Abyss, multithreading provided me an advantage when simulating more than 3 rope instances at a time. I found best efficiency when assigning about two simulations to each worker thread.
    - Performance is heavily dependent on your rope settings, so tweak and measure accordingly.

Don't forget to check Godot's internal documention if you aren't sure what something does! Tip: F1 is the default hotkey for the Help menu.

![](images/godot-internal-docs.png)
