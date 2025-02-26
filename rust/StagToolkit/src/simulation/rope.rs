use glam::{vec3, vec4, Vec3, Vec4};

/// Data for managing a simulated rope.
#[derive(Clone)]
pub struct RopeData {
    /// Number of points in the rope.
    pub point_count: u32,
    /// Ideal distance between points in the rope.
    pub distance_per_point: f32,
    /// Spring constant of the rope.
    pub spring_constant: f32,
    /// Constant acceleration applied to the rope.
    pub acceleration: Vec3,

    /// All attached binding global positions, and corresponding rope parameter.
    pub bindings: Vec<Vec4>,

    /// All current rope positions.
    pub points: Vec<Vec3>,
}

impl RopeData {
    /// Generates a new RopeData struct.
    pub fn new() -> Self {
        Self {
            point_count: 2,
            distance_per_point: 1.0,
            spring_constant: 5000.0,
            acceleration: vec3(0.0, -9.81, 0.0),
            bindings: vec![vec4(0.0, 0.0, 0.0, 0.0), vec4(1.0, 0.0, 0.0, 1.0)],
            points: vec![vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)],
        }
    }
}

impl Default for RopeData {
    fn default() -> Self {
        Self::new()
    }
}
