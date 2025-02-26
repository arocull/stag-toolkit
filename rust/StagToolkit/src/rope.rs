use crate::{math::types::ToVector3, simulation::rope::RopeData};
use godot::{
    classes::{Mesh, MeshInstance3D, ShaderMaterial},
    prelude::*,
};

const GROUP_NAME_ROPEBINDING: &str = "StagToolkit_SimulatedRopeBinding";

/// Settings for a simulated rope class.
#[derive(GodotClass)]
#[class(init,base=Resource)]
pub struct SimulatedRopeSettings {
    /// Ideal number of meters between each point on the rope.
    /// The amount of points on the rope is rounded based on the rope's ideal length divided by this amount.
    #[var(get, set = set_point_distance)]
    #[export(range = (0.0, 2.0, 0.01, or_greater, suffix="m"))]
    #[init(val = 0.25)]
    point_distance: f32,

    /// Spring constant of the rope.
    /// For every unit of length overstretched: that distance squared, times this constant, is applied in force.
    #[var(get, set = set_spring_constant)]
    #[export]
    #[init(val = 5000.0)]
    spring_constant: f32,

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
    /// The only parameter passed to the shader is `points`, an array of rope point positions.
    /// This material is always applied to the first surface slot of the rope mesh.
    #[export]
    #[init(val=None)]
    render_material: Option<Gd<ShaderMaterial>>,

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

    base: Base<Resource>,
}

#[godot_api]
impl SimulatedRopeSettings {
    // TODO: use self.signals() when typed signal implementation is released

    #[func]
    fn set_point_distance(&mut self, new_point_distance: f32) {
        self.point_distance = new_point_distance;
        self.base_mut().emit_signal("simulation_changed", &[]);
    }

    #[func]
    fn set_spring_constant(&mut self, new_spring_constant: f32) {
        self.spring_constant = new_spring_constant;
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

    data: RopeData,

    base: Base<Node3D>,
}

#[godot_api]
impl INode3D for SimulatedRope {
    fn ready(&mut self) {
        self.initialize_simulation();
        self.initialize_render();
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
            data.point_count = (self.ideal_length / settings.point_distance).round() as u32;
            data.distance_between_points = (self.ideal_length / (data.point_count as f32)).powi(2);
            data.spring_constant = settings.spring_constant;

            data.bindings.clear();

            self.data = data;
        }

        self.base_mut().set_physics_process(false);
        godot_error!("Failed to prepare SimulatedRope: no settings provided.");
    }

    /// Regenerates the rope mesh based on its given render settings.
    #[func]
    pub fn initialize_render(&mut self) {
        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();

            let mut mesh = self.fetch_mesh_instance();

            if let Some(mesh_data) = settings.render_mesh.clone() {
                mesh.set_mesh(&mesh_data);
            }
            if let Some(mesh_shader) = settings.render_material.clone() {
                mesh.set_surface_override_material(0, &mesh_shader);
            }

            mesh.set_layer_mask(settings.render_layers);
        }
    }

    /// Regenerates the rope physics queries based on its given collision settings.
    #[func]
    pub fn initialize_collision(&mut self) {}

    /// Fetches the rope mesh instance, creating one if not provided.
    #[func]
    pub fn fetch_mesh_instance(&mut self) -> Gd<MeshInstance3D> {
        if let Some(node) = self.base().get_node_or_null("mesh_rope") {
            if let Ok(mesh) = node.try_cast::<MeshInstance3D>() {
                return mesh;
            }
        }

        let mut mesh = MeshInstance3D::new_alloc();
        mesh.set_name("mesh_rope");
        self.base_mut().add_child(&mesh);
        mesh
    }

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
