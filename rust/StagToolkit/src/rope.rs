use crate::simulation::rope::RopeData;
use godot::prelude::*;

const GROUP_NAME_ROPEBINDING: &str = "StagToolkit_SimulatedRopeBinding";

/// Settings for a simulated rope class.
#[derive(GodotClass)]
#[class(init,base=Resource)]
pub struct SimulatedRopeSettings {
    /// Ideal number of meters between each point on the rope.
    /// The amount of points on the rope is rounded based on the rope's ideal length divided by this amount.
    #[export(range = (0.0, 2.0, 0.01, or_greater, suffix="m"))]
    #[init(val = 0.25)]
    point_distance: f32,

    /// Spring constant of the rope.
    /// For every unit of length overstretched: that distance squared, times this constant, is applied in force.
    #[export]
    #[init(val = 5000.0)]
    spring_constant: f32,

    base: Base<Resource>,
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
        self.reinitialize();
    }
}

#[godot_api]
impl SimulatedRope {
    /// Regenerates internal rope data based on the given simulation settings.
    pub fn reinitialize(&mut self) {
        if let Some(settings) = self.settings.clone() {
            let settings = settings.bind();

            // Generate new rope data and apply settings
            let mut data = RopeData::new();
            data.point_count = (self.ideal_length / settings.point_distance).round() as u32;
            data.distance_per_point = self.ideal_length / (data.point_count as f32);
            data.spring_constant = settings.spring_constant;

            data.bindings.clear();

            self.data = data;
        }

        self.base_mut().set_physics_process(false);
        godot_error!("Failed to prepare SimulatedRope: no settings provided.");
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
