use glam::{vec3, vec4, Vec3, Vec4, Vec4Swizzles};

/// Returns a tuple of values A and B, constrainted within the given distance from each other.
///
/// As described in [Robert Bodea's rope simulation article](https://owlree.blog/posts/simulating-a-rope.html).
pub fn jakobsen_constraint(a: Vec3, b: Vec3, ideal_distance: f32) -> (Vec3, Vec3) {
    // Half the change in distance we need in order to meet the constraint.
    let distance_offset = (a.distance(b) - ideal_distance) * 0.5;
    // Direction from previous point to this one, multiplied by offset distance.
    let offset = (b - a).normalize() * distance_offset;
    (a + offset, b - offset)
}

/// Data for managing a simulated rope.
///
/// I use techniques described in [Robert Bodea's rope simulation article](https://owlree.blog/posts/simulating-a-rope.html).
#[derive(Clone)]
pub struct RopeData {
    /// Number of points in the rope.
    pub point_count: u32,
    /// Ideal squared distance between points in the rope.
    pub distance_between_points: f32,
    /// Spring constant of the rope.
    pub spring_constant: f32,
    /// Constant acceleration applied to the rope.
    pub acceleration: Vec3,

    /// All attached binding positions, and corresponding rope parameter.
    pub bindings: Vec<Vec4>,

    /// All current rope positions.
    pub points: Vec<Vec3>,

    /// All previous rope positions.
    pub previous_points: Vec<Vec3>,
}

impl RopeData {
    /// Generates a new RopeData struct.
    pub fn new() -> Self {
        Self {
            point_count: 2,
            distance_between_points: 1.0,
            spring_constant: 5000.0,
            acceleration: vec3(0.0, -9.81, 0.0),
            bindings: vec![vec4(0.0, 0.0, 0.0, 0.0), vec4(1.0, 0.0, 0.0, 1.0)],
            points: vec![vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)],
            previous_points: vec![vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0)],
        }
    }

    /// Returns the point index for the given binding location (between 0 and 1).
    pub fn bind_index(&self, param: f32) -> usize {
        (param.clamp(0.0, 1.0) * ((self.bindings.len() - 1) as f32)).round() as usize
    }

    /// Steps the simulation forward by many X seconds using Verlet integration.
    /// Does NOT apply constraints.
    pub fn step(&mut self, delta_time_squared: f32) {
        for idx in 0..self.points.len() {
            let p = self.points[idx];
            self.points[idx] =
                2.0 * p - self.previous_points[idx] + delta_time_squared * self.acceleration;
            self.previous_points[idx] = p;
        }
    }

    /// Constrains the system X many times, snapping the system back to bound points.
    /// Uses the Jakobsen Method.
    ///
    /// TODO: should this fill a new set of points each iteration instead of operating on the same dataset?
    pub fn constrain(&mut self, steps: u32) {
        // Pre-compute binding indices
        let mut bind_indices: Vec<usize> = Vec::with_capacity(self.bindings.len());
        for b in self.bindings.iter() {
            bind_indices.push(self.bind_index(b.w));
        }

        // Run many iterations
        for _ in 0..steps {
            // Force points towards/away from each other to meet the constraint
            for idx in 1..self.points.len() {
                // Constrain with previous point
                (self.points[idx], self.points[idx - 1]) = jakobsen_constraint(
                    self.points[idx],
                    self.points[idx - 1],
                    self.distance_between_points,
                );
            }

            // Enforce binding positions, if any are present
            for idx in 0..self.bindings.len() {
                self.points[bind_indices[idx]] = self.bindings[idx].xyz();
            }
        }
    }
}

impl Default for RopeData {
    fn default() -> Self {
        Self::new()
    }
}
