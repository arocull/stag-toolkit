use std::collections::HashMap;

use crate::{math::types::ToVector3, simulation::rope::RopeData};
use glam::{vec4, Vec3, Vec4};
use godot::{
    classes::{
        Engine, Mesh, MeshInstance3D, ProjectSettings, ResourceLoader, RigidBody3D, ShaderMaterial,
    },
    init::is_main_thread,
    prelude::*,
};

const GROUP_NAME_ROPE: &str = "StagToolkit_SimulatedRope";
const GROUP_NAME_ROPEBINDING: &str = "StagToolkit_SimulatedRopeBinding";
const MESH_NAME: &str = "mesh_rope";

/// Settings for a simulated rope class.
/// @experimental
#[derive(GodotClass)]
#[class(init,base=Resource,tool)]
pub struct SimulatedRopeSettings {
    /// Ideal number of meters between each point on the rope.
    /// The amount of points on the rope is rounded based on the rope's ideal length divided by this amount.
    #[var(get, set = set_simulation_point_distance)]
    #[export(range = (0.0, 2.0, 0.01, or_greater, suffix="m"))]
    #[init(val = 0.25)]
    simulation_point_distance: f32,

    /// Spring constant of the rope.
    /// For every unit of length overstretched: that distance squared, times this constant, is applied in force.
    #[var(get, set = set_simulation_spring_constant)]
    #[export]
    #[init(val = 5000.0)]
    simulation_spring_constant: f32,

    /// Number of iterations for applying a Jakobsen constraint (ensures each point is within the `simulation_point_distance`).
    #[var(get, set = set_simulation_constraint_iterations)]
    #[export(range = (0.0, 500.0, 1.0))]
    #[init(val = 30)]
    simulation_constraint_iterations: u32,

    /// Whether or not to automatically call `tick_simulation` on the physics process tick.
    ///
    /// If this is `false`, **the simulation is not ticked at all**, and is expected to be ticked manually by the user.
    #[export]
    #[init(val = true)]
    simulation_tick_on_physics: bool,

    /// Whether to generate a corresponding [MeshInstance3D] for visualizing the rope.
    #[export]
    #[init(val = true)]
    render: bool,

    /// What render layers the mesh should be on.
    #[export(flags_3d_render)]
    #[init(val = 1)]
    render_layers: u32,

    /// What mesh to use for rendering.
    #[export]
    #[init(val=None)]
    render_mesh: Option<Gd<Mesh>>,

    /// What shader material to use for rendering.
    ///
    /// This material is always applied to the first surface slot of the rope mesh.
    ///
    /// A duplicate of this material is made upon calling `initialize_render` for each [SimulatedRope],
    /// to prevent conflicting parameters.
    ///
    /// The only parameter passed to the shader is `points`, an array of rope point positions.
    #[export]
    #[init(val=None)]
    render_material: Option<Gd<ShaderMaterial>>,

    #[export]
    #[init(val="points".into())]
    render_parameter_points: GString,

    #[export]
    #[init(val="point_count".into())]
    render_parameter_point_count: GString,

    // Updates the rope AABB after this many seconds.
    #[export(range=(0.0,1.0,0.01,or_greater,suffix="s"))]
    #[init(val = 0.2)]
    render_aabb_update_rate: f64,

    /// All [SimulatedRope] nodes using these settings will automatically set their `process_priority` to this value.
    /// It is reccomended this is greater than the `collision_process_priority` in cases where collision is utilized.
    #[export]
    #[init(val = 2)]
    render_process_priority: i32,

    /// Whether to perform raycasts to attempt collision with the 3D environment.
    #[export]
    #[init(val = true)]
    collision_raycasts: bool,

    /// When performing collision checks,
    #[export(flags_3d_physics)]
    #[init(val = 1)]
    collision_mask: u32,

    /// Rope points are forced to be this distance from any collision point.
    #[export(range=(0.0,1.0,0.001,suffix="m"))]
    #[init(val = 0.05)]
    collision_offset: f32,

    /// All [SimulatedRope] nodes using these settings will automatically set their `physics_process_priority` to this value.
    #[export]
    #[init(val = 1)]
    collision_process_priority: i32,

    base: Base<Resource>,
}

#[godot_api]
impl SimulatedRopeSettings {
    // TODO: use self.signals() when typed signal implementation is released

    #[func]
    fn set_simulation_point_distance(&mut self, new_point_distance: f32) {
        self.simulation_point_distance = new_point_distance.max(0.01);
        self.base_mut().emit_signal("simulation_changed", &[]);
    }

    #[func]
    fn set_simulation_spring_constant(&mut self, new_spring_constant: f32) {
        self.simulation_spring_constant = new_spring_constant.max(0.0);
        self.base_mut().emit_signal("simulation_changed", &[]);
    }

    #[func]
    fn set_simulation_constraint_iterations(&mut self, new_constraint_iterations: i64) {
        self.simulation_constraint_iterations =
            (new_constraint_iterations.unsigned_abs() as u32).max(1);
        self.base_mut().emit_signal("simulation_changed", &[]);
    }

    /// Emitted when any simulation setting changes, requiring re-generation of the internal rope data.
    #[signal]
    fn simulation_changed();

    /// Emitted when any render setting changes, requiring a re-generation of the rope mesh.
    #[signal]
    fn render_changed();

    /// Emitted when any physics setting changes, requiring a re-generation of RayQuery parameters.
    #[signal]
    fn physics_changed();
}

/// Godot interface for managing a simulated rope.
/// @experimental
#[derive(GodotClass)]
#[class(init,base=Node3D,tool)]
pub struct SimulatedRope {
    /// Ideal length of the rope.
    #[var(get, set = set_ideal_length)]
    #[export(range = (0.1, 100.0, or_greater))]
    #[init(val = 25.0)]
    ideal_length: f32,

    /// Settings for the rope.
    #[var(get, set = set_settings)]
    #[export]
    #[init(val=None)]
    settings: Option<Gd<SimulatedRopeSettings>>,

    /// A clone of the provided shader material in render settings. Handled automatically.
    #[var]
    #[init(val=None)]
    shader: Option<Gd<ShaderMaterial>>,

    /// Fallback settings for the rope, if none is provided by user. Handled automatically.
    #[init(val=None)]
    settings_fallback: Option<Gd<SimulatedRopeSettings>>,

    /// Whether or not to automatically perform simulation ticks.
    #[init(val = true)]
    do_simulation_tick: bool,

    /// Internal, simulated rope data.
    data: RopeData,

    /// Attached binding IDs, with a corresponding Vec4 with XYZ position, and rope parameter W.
    #[init(val =(HashMap::<i64, Vec4>::new()))]
    bindings: HashMap<i64, Vec4>,

    #[init(val = 0.0)]
    aabb_timer: f64,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for SimulatedRope {
    fn ready(&mut self) {
        // Add to node group for rope
        self.base_mut()
            .add_to_group_ex(GROUP_NAME_ROPE)
            .persistent(true)
            .done();

        self.initialize_simulation();
        self.initialize_render();
        self.initialize_collision();
    }

    fn process(&mut self, delta: f64) {
        self.tick_render();

        // Update AABB after X many seconds
        self.aabb_timer += delta;
        if let Some(settings) = self.get_settings() {
            if self.aabb_timer >= settings.bind().render_aabb_update_rate {
                self.tick_render_aabb();
            }
        }
    }

    fn physics_process(&mut self, delta: f64) {
        if self.do_simulation_tick {
            self.tick_simulation(delta);
            // godot_print!("rope simulation tick: {0}\t{1}", delta, self.data.points.len());
        }

        self.tick_collision();
    }
}

#[godot_api]
impl SimulatedRope {
    #[func]
    fn set_settings(&mut self, new_settings: Option<Gd<SimulatedRopeSettings>>) {
        let init_sim_callable = &self.base_mut().callable("initialize_simulation");

        // Unbind our simulation reset from the old settings
        if let Some(mut settings) = self.settings.clone() {
            if settings.is_connected("simulation_changed", init_sim_callable) {
                settings.disconnect("simulation_changed", init_sim_callable);
            }
        }

        // Bind our simulation reset to the new settings
        if let Some(mut settings) = new_settings.clone() {
            settings.connect("simulation_changed", init_sim_callable);

            // Clear out internal fallback settings if we don't need them
            self.settings_fallback = None;
        }

        self.settings = new_settings;
    }

    /// Sets the ideal length of the rope.
    /// **Does not regenerate the rope**, but instead changes the ideal distance between rope points.
    /// This is in case the user may want to elongate the rope without restarting the simulation.
    /// To regenerate the rope during runtime, call `initialize_simulation`.
    #[func]
    pub fn set_ideal_length(&mut self, new_ideal_length: f32) {
        self.ideal_length = new_ideal_length.max(0.1);

        // TODO: can we update the simulation without changing the number of points?
        self.data.distance_between_points = self.ideal_length / (self.data.point_count as f32);
        // self.initialize_simulation();
    }

    /// Sets or replaces a bind on the rope with the corresponding `bind_id`.
    /// `position` is where the bind is placed in the [SimulatedRope]'s local space.
    /// `rope_factor` is what part of the rope should be constrained to the bind, in a range between 0 and 1.
    /// `rope_factor` is automatically scaled and rounded to a corresponding point index during simulation.
    #[func]
    fn bind_set(&mut self, bind_id: i64, position: Vector3, rope_factor: f32) {
        self.bindings.insert(
            bind_id,
            vec4(position.x, position.y, position.z, rope_factor),
        );
    }

    /// Removes a bind from the cache.
    #[func]
    fn bind_erase(&mut self, bind_id: i64) {
        self.bindings.remove(&bind_id);
    }

    /// Clears the binding cache.
    /// Note that this may unintentionally disconnect any [SimulatedRopeBinding] until their next tick.
    #[func]
    fn bind_clear(&mut self) {
        self.bindings.clear();
    }

    /// Regenerates internal rope data based on its given simulation settings.
    #[func]
    pub fn initialize_simulation(&mut self) {
        let settings_resource = self.fetch_settings();
        let settings = settings_resource.bind();

        self.do_simulation_tick = settings.simulation_tick_on_physics;

        // Generate new rope data and apply settings
        let mut data = RopeData::new(self.ideal_length, settings.simulation_point_distance);

        data.spring_constant = settings.simulation_spring_constant;
        data.constraint_iterations = settings.simulation_constraint_iterations;

        self.data = data;
    }

    /// Regenerates the rope mesh based on its given render settings.
    #[func]
    pub fn initialize_render(&mut self) {
        // Do nothing outside of tree
        if !self.base().is_inside_tree() {
            return;
        }

        let settings_resource = self.fetch_settings();
        let settings = settings_resource.bind();

        self.shader = None; // Clear out shader reference so it's culled by Godot

        // If rendering is disabled, remove any potential meshes, and exit early
        if !settings.render {
            if let Some(mut node) = self.base().get_node_or_null(MESH_NAME) {
                node.queue_free();
            }
            return;
        }

        let mut mesh = self.fetch_mesh_instance();

        if let Some(mesh_data) = settings.render_mesh.clone() {
            mesh.set_mesh(&mesh_data);
        }

        // If we have an available shader
        if let Some(base_shader) = settings.render_material.clone() {
            // Make a clone of the shader material so we can freely modifiy its parameters
            if let Some(unique_shader_resource) = base_shader.duplicate() {
                if let Ok(unique_shader) = unique_shader_resource.try_cast::<ShaderMaterial>() {
                    self.shader = Some(unique_shader.clone());
                    mesh.set_surface_override_material(0, &unique_shader);
                }
            }
        }

        mesh.set_layer_mask(settings.render_layers);

        // Update preview immediately
        self.tick_render();
        self.tick_render_aabb();
    }

    /// Regenerates the rope physics queries based on its given collision settings.
    #[func]
    pub fn initialize_collision(&mut self) {
        let settings_resource = self.fetch_settings();
        let settings = settings_resource.bind();

        self.do_simulation_tick = settings.simulation_tick_on_physics;

        self.base_mut()
            .set_physics_process_priority(settings.collision_process_priority);
    }

    /// Fetches the simulated rope settings, using fallbacks if none provided.
    /// TODO: load project default if provided.
    #[func]
    pub fn fetch_settings(&mut self) -> Gd<SimulatedRopeSettings> {
        // Default to our existing settings if provided
        if let Some(settings) = self.settings.clone() {
            return settings;
        }

        // If no settings provided, use fallback if they exist
        if let Some(settings) = self.settings_fallback.clone() {
            return settings;
        }

        // If no fallback exists, attempt to fetch one from project settings
        // Only call this on main-thread for garuanteed thread safety while handling resources
        if is_main_thread() {
            let project_settings = ProjectSettings::singleton();
            let defaults_path = project_settings
                .get_setting_ex("addons/stag_toolkit/simulated_rope/default_settings")
                .default_value(&"".to_variant())
                .done();
            let defaults_path: GString = defaults_path.to();

            // Attempt to load default if not empty
            let mut resource_loader = ResourceLoader::singleton();
            if !defaults_path.is_empty() && resource_loader.exists(&defaults_path) {
                // Load the settings from the path
                let default_settings = try_load::<SimulatedRopeSettings>(&defaults_path);

                // Ensure the settings are okay
                if let Ok(new_settings) = default_settings {
                    self.settings_fallback = Some(new_settings.clone());
                    return new_settings;
                // Otherwise, throw a warning to let the developer know
                } else if let Err(bad_settings) = default_settings {
                    godot_warn!("SimulatedRope failed to load default SimulatedRopeSettings resource from project settings (addons/stag_toolkit/simulated_rope/default_settings): {0}", bad_settings);
                }
            }
        }

        // Otherwise, create fallback settings
        let settings = SimulatedRopeSettings::new_gd();
        self.settings_fallback = Some(settings.clone());

        settings
    }

    /// Fetches the rope mesh instance, creating one if not provided.
    #[func]
    pub fn fetch_mesh_instance(&mut self) -> Gd<MeshInstance3D> {
        if let Some(node) = self.base().get_node_or_null(MESH_NAME) {
            if let Ok(mesh) = node.try_cast::<MeshInstance3D>() {
                return mesh;
            }
        }

        let mut mesh = MeshInstance3D::new_alloc();
        mesh.set_name(MESH_NAME);
        self.base_mut().add_child(&mesh);
        mesh
    }

    /// Ticks the rope simulation forward by `delta` seconds.
    /// This method is thread-safe (ideally).
    #[func]
    pub fn tick_simulation(&mut self, delta: f64) {
        // Generate bind map for faster computation
        let bind_map = self.data.unique_bind_map(&self.bindings);

        // Compute tension data
        self.data.tension(&bind_map);

        // First, step simulation
        self.data.step(delta, &bind_map);

        // Apply constraints
        self.data.constrain(&bind_map);
    }

    /// Ticks the rope render, updating shader parameters and corresponding AABB.
    /// TODO: should we have data interpolation?
    #[func]
    pub fn tick_render(&mut self) {
        // Update shader parameters
        if let Some(mut shader) = self.shader.clone() {
            let settings_resource = self.fetch_settings();
            let settings = settings_resource.bind();

            let pts: PackedVector3Array = self.data.points.clone().to_vector3();
            shader.set_shader_parameter(settings.render_parameter_points.arg(), &pts.to_variant());
            shader.set_shader_parameter(
                settings.render_parameter_point_count.arg(),
                &self.data.point_count.to_variant(),
            );
        }
    }

    /// Updates the AABB on the rope render.
    #[func]
    pub fn tick_render_aabb(&mut self) {
        let mut mesh = self.fetch_mesh_instance();
        mesh.set_custom_aabb(self.get_aabb());
        self.aabb_timer = 0.0;
    }

    /// Ticks the rope collision, attempting to collide with terrain.
    /// Must be run on physics tick.
    #[func]
    pub fn tick_collision(&mut self) {}

    /// Computes and returns an enclosing [AABB] for the rope.
    #[func]
    pub fn get_aabb(&self) -> Aabb {
        let mut aabb = Aabb::new(self.data.points[0].to_vector3(), Vector3::ZERO);

        for i in 1..self.data.points.len() {
            aabb = aabb.expand(self.data.points[i].to_vector3());
        }

        aabb
    }

    /// Returns the tension force, in global space, at the given point on the rope within the range [0,1].
    #[func]
    pub fn get_tension_force_at(&self, factor: f32) -> Vector3 {
        let basis = self.base().get_global_basis();

        let idx = self.data.bind_index(factor);
        let force_local: Vector3 = self.data.force(idx).to_vector3();

        basis.inverse() * force_local
    }

    /// Returns the rope factor of the nearest rope point at the given global space position.
    #[func]
    pub fn get_rope_factor(&self, position: Vector3) -> f32 {
        let local: Vec3 = self.base().to_local(position).to_vector3();

        let mut closest_idx: usize = 0;
        let mut closest_dist: f32 = f32::MAX;
        for (idx, pt) in self.data.points.iter().enumerate() {
            let d = local.distance_squared(*pt);
            if local.distance_squared(*pt) < closest_dist {
                closest_dist = d;
                closest_idx = idx;
            }
        }

        self.data.bind_factor(closest_idx)
    }

    /// Returns the global space rope position at the given rope factor.
    #[func]
    pub fn get_rope_position(&self, factor: f64) -> Vector3 {
        let idx = self.data.bind_index(factor as f32);
        self.base().to_global(self.data.points[idx].to_vector3())
    }

    /// Returns the distance to the nearest rope point at the given global space position.
    #[func]
    pub fn get_rope_distance(&self, position: Vector3) -> f32 {
        let local: Vec3 = self.base().to_local(position).to_vector3();

        let mut closest_dist: f32 = f32::MAX;
        for pt in self.data.points.iter() {
            let d = local.distance_squared(*pt);
            if local.distance_squared(*pt) < closest_dist {
                closest_dist = d;
            }
        }

        closest_dist.sqrt()
    }

    /// Returns the amount of slack in the rope, in a range of 0 to 1, at the given rope factor.
    #[func]
    pub fn get_rope_slack(&self, factor: f32) -> f32 {
        self.data.slack(self.data.bind_index(factor))
    }

    /// Returns the AVERAGED forward direction ("forward" meaning the direction FROM a factor of 0 TOWARD a factor of 1),
    /// sampling all points between the given `factor` and `factor + factor width`.
    /// If desired, factor width can be determined via a sample distance and the rope's length: `sample_distance / rope.ideal_length`.
    #[func]
    pub fn get_rope_slide_direction(&self, factor: f32, factor_width: f32) -> Vector3 {
        let bind_min: usize = self.data.bind_index(factor.clamp(0.0, 1.0)).max(1);
        let bind_max: usize = self
            .data
            .bind_index((factor + factor_width).clamp(0.0, 1.0));

        let mut dir: Vec3 = Vec3::ZERO;

        for i in bind_min..bind_max {
            dir += (self.data.points[i - 1] - self.data.points[i]).normalize_or_zero();
        }

        (dir / (bind_max - bind_min) as f32)
            .normalize_or_zero()
            .to_vector3()
    }
}

/// Attaches to a simulated rope, and provides force readings from it.
/// Automatically applies force readings to the parent RigidBody, if enabled.
/// @experimental
#[derive(GodotClass)]
#[class(init,base=Node3D,tool)]
pub struct SimulatedRopeBinding {
    /// A simulated rope to attach this binding to.
    #[var(get, set = set_bind_to)]
    #[export]
    #[init(val=None)]
    bind_to: Option<Gd<SimulatedRope>>,

    /// Where on the rope, as a percentage of its length ("rope factor"), this binding is attached.
    #[var(get, set = set_bind_at)]
    #[export(range = (0.0, 1.0, 0.00001))]
    #[init(val = 0.0)]
    bind_at: f32,

    /// Scales the spring factor of the rope by this amount when providing force estimates.
    #[export(range = (0.0,10.0,0.001,or_greater))]
    #[init(val = 1.0)]
    spring_constant_multiplier: f32,

    /// What tick to update the [SimulatedRope]'s bound position on.
    #[var(get, set = set_update_tick)]
    #[export(enum = (Disabled = 0, Process = 1, PhysicsProcess = 2))]
    #[init(val = 2)]
    update_tick: i32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for SimulatedRopeBinding {
    fn ready(&mut self) {
        // Add to node group for rope bindings
        self.base_mut()
            .add_to_group_ex(GROUP_NAME_ROPEBINDING)
            .persistent(true)
            .done();

        // Update any bindings immediately
        self.update_bind();

        let update_tick = self.update_tick;
        self.base_mut().set_process(update_tick == 1);
        self.base_mut().set_physics_process(true);
    }

    fn exit_tree(&mut self) {
        if let Some(mut rope) = self.bind_to.clone() {
            rope.bind_mut().bind_erase(self.get_bind_id());
        }
    }

    fn process(&mut self, _delta: f64) {
        self.update_bind();
    }

    fn physics_process(&mut self, _delta: f64) {
        // TODO: apply rope simulation forces
        if !Engine::singleton().is_editor_hint() {
            if let Some(mut rigid) = self.get_rigid_body() {
                if let Some(rope) = self.get_bind_to() {
                    let rope = rope.bind();

                    let force: Vector3 =
                        rope.get_tension_force_at(self.bind_at) * self.spring_constant_multiplier;

                    let pos =
                        self.base().get_global_position() - rigid.clone().get_global_position();

                    rigid.apply_force_ex(force).position(pos).done();
                }
            }
        }

        if self.update_tick == 2 {
            self.update_bind();
        }
    }
}

#[godot_api]
impl SimulatedRopeBinding {
    #[func]
    fn set_bind_to(&mut self, new_bind_to: Option<Gd<SimulatedRope>>) {
        let id = self.base().instance_id().to_i64();

        // If we had an existing bind, remove it
        if let Some(mut rope) = self.bind_to.clone() {
            rope.bind_mut().bind_erase(id);
        }

        self.bind_to = new_bind_to;

        if self.base().is_inside_tree() {
            self.update_bind();
        }
    }

    #[func]
    fn set_bind_at(&mut self, new_bind_at: f32) {
        self.bind_at = new_bind_at.clamp(0.0, 1.0);
        if self.base().is_inside_tree() {
            self.update_bind();
        }
    }

    #[func]
    fn set_update_tick(&mut self, new_update_tick: i32) {
        self.update_tick = new_update_tick;
        self.base_mut().set_process(new_update_tick == 1);
    }

    /// Returns this rope binding's bind ID.
    #[func]
    fn get_bind_id(&self) -> i64 {
        self.base().instance_id().to_i64()
    }

    /// Updates the bind settings on this RopeBinding's corresponding rope.
    #[func]
    fn update_bind(&mut self) {
        if let Some(mut rope) = self.bind_to.clone() {
            let pos = rope.to_local(self.base().get_global_position());
            rope.bind_mut()
                .bind_set(self.get_bind_id(), pos, self.bind_at);
        }
    }

    /// Recursively walks up tree until a RigidBody3D is found, returning it, or `null` if not found.
    #[func]
    fn get_rigid_body(&self) -> Option<Gd<RigidBody3D>> {
        Self::get_rigid_body_recursive(self.base().get_parent_node_3d())
    }

    fn get_rigid_body_recursive(node: Option<Gd<Node3D>>) -> Option<Gd<RigidBody3D>> {
        if let Some(parent) = node {
            if let Ok(rigid_body) = parent.clone().try_cast::<RigidBody3D>() {
                return Some(rigid_body);
            } else {
                return Self::get_rigid_body_recursive(parent.get_parent_node_3d());
            }
        }
        None
    }

    /// Slides the binding's factor forward or back by the given amount, halting before the next binding.
    #[func]
    fn slide_bind_at(&mut self, factor_amount: f32) {
        let mut new_factor: f32;
        if let Some(rope) = self.bind_to.clone() {
            let sim = rope.bind();

            let idx_current = sim.data.bind_index(self.bind_at);

            // Get bounds for binding
            let bind_map = sim.data.unique_bind_map(&sim.bindings);
            let (smallest, has_smallest, largest, has_largest) = sim
                .data
                .get_surrounding_bind_indices(idx_current, &bind_map);

            // Construct new bind index
            new_factor = self.bind_at + factor_amount;
            let mut new_bind_index = sim.data.bind_index(new_factor);

            // Clamp bind index to the given bounds, if present
            if has_smallest {
                new_bind_index = new_bind_index.max(smallest + 1);
            }
            if has_largest {
                new_bind_index = new_bind_index.min(largest - 1);
            }

            // Finally, update bind
            new_factor = sim.data.bind_factor(new_bind_index);

            // godot_print!("Smallest: {0} {1}\tLargest: {2} {3}\tFinal: {4} {5}", smallest, has_smallest, largest, has_largest, new_bind_index, new_factor);
        } else {
            new_factor = (self.bind_at + factor_amount).clamp(0.0, 1.0);
        }

        self.set_bind_at(new_factor);
    }
}
