use crate::{math::types::ToVector3, simulation::rope::RopeData};
use godot::{
    classes::{Mesh, MeshInstance3D, ShaderMaterial},
    prelude::*,
};

const GROUP_NAME_ROPEBINDING: &str = "StagToolkit_SimulatedRopeBinding";
const MESH_NAME: &str = "mesh_rope";

/// Settings for a simulated rope class.
/// @experimental
#[derive(GodotClass)]
#[class(init,base=Resource)]
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
    #[export(range = (0.0, 128.0, 1.0, or_greater))]
    #[init(val = 10)]
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
    #[export(range = (0.1, 100.0, or_greater))]
    #[init(val = 25.0)]
    ideal_length: f32,

    /// Settings for the rope.
    #[export]
    #[init(val=None)]
    settings: Option<Gd<SimulatedRopeSettings>>,

    /// A clone of the provided shader material in render settings. Handled automatically.
    #[var]
    #[init(val=None)]
    shader: Option<Gd<ShaderMaterial>>,

    /// Whether or not to automatically perform simulation ticks.
    #[init(val = true)]
    do_simulation_tick: bool,

    data: RopeData,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for SimulatedRope {
    fn ready(&mut self) {
        self.initialize_simulation();
        self.initialize_render(); // TODO: should this be deferred?
        self.initialize_collision();
    }

    fn process(&mut self, _delta: f64) {
        self.tick_render();
    }

    fn physics_process(&mut self, delta: f64) {
        if self.do_simulation_tick {
            self.tick_simulation(delta);
        }

        self.tick_collision();
    }
}

#[godot_api]
impl SimulatedRope {
    /// Regenerates internal rope data based on its given simulation settings.
    #[func]
    pub fn initialize_simulation(&mut self) {
        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();

            // Generate new rope data and apply settings
            let mut data = RopeData::new();
            data.point_count =
                (self.ideal_length / settings.simulation_point_distance).round() as u32;
            data.distance_between_points = (self.ideal_length / (data.point_count as f32)).powi(2);
            data.spring_constant = settings.simulation_spring_constant;
            data.constraint_iterations = settings.simulation_constraint_iterations;

            data.bindings.clear();

            self.data = data;
        }

        self.base_mut().set_physics_process(false);
        godot_error!("Failed to prepare SimulatedRope: no settings provided.");
    }

    /// Regenerates the rope mesh based on its given render settings.
    #[func]
    pub fn initialize_render(&mut self) {
        // Do nothing outside of tree
        if !self.base().is_inside_tree() {
            return;
        }

        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();

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
        }
    }

    /// Regenerates the rope physics queries based on its given collision settings.
    #[func]
    pub fn initialize_collision(&mut self) {
        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();

            self.do_simulation_tick = settings.simulation_tick_on_physics;

            self.base_mut()
                .set_physics_process_priority(settings.collision_process_priority);
        }
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
        self.data.step(delta as f32);
        self.data.constrain();
    }

    /// Ticks the rope render, updating shader parameters and corresponding AABB.
    /// TODO: should we have data interpolation?
    #[func]
    pub fn tick_render(&mut self) {
        // Update shader parameters
        if let Some(mut shader) = self.shader.clone() {
            let pts: PackedVector3Array = self.data.points.clone().to_vector3();
            shader.set_shader_parameter("points", &pts.to_variant());
        }
    }

    /// Ticks the rope collision, attempting to collide with terrain.
    /// Must be run on physics tick.
    #[func]
    pub fn tick_collision(&mut self) {}

    /// Computes and returns an enclosing [AABB] for the rope.
    #[func]
    pub fn aabb(&self) -> Aabb {
        let mut aabb = Aabb::new(self.data.points[0].to_vector3(), Vector3::ZERO);

        for i in 1..self.data.points.len() {
            aabb = aabb.expand(self.data.points[i].to_vector3());
        }

        aabb
    }
}

/// Attaches to a simulated rope, and provides force readings from it.
/// Automatically applies force readings to the parent RigidBody, if enabled.
/// @experimental
#[derive(GodotClass)]
#[class(init,base=Node3D,tool)]
pub struct SimulatedRopeBinding {
    /// A simulated rope to attach this binding to.
    #[export]
    #[init(val=None)]
    bind_to: Option<Gd<SimulatedRope>>,

    /// Where on the rope, as a percentage of its length, this binding is attached.
    #[export(range = (0.0, 1.0, 0.001, suffix="%"))]
    #[init(val = 0.0)]
    bind_at: f32,

    /// Scales the spring factor of the rope by this amount when providing force estimates.
    #[export(range = (0.0,10.0,0.001,or_greater))]
    #[init(val = 1.0)]
    spring_factor_multiplier: f32,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for SimulatedRopeBinding {
    fn ready(&mut self) {
        self.base_mut()
            .add_to_group_ex(GROUP_NAME_ROPEBINDING)
            .persistent(true)
            .done();
    }
}

#[godot_api]
impl SimulatedRopeBinding {}
