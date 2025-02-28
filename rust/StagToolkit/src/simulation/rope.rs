use std::collections::HashMap;

use glam::{vec3, Vec3, Vec4, Vec4Swizzles};

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

/// Returns a one-sided Jakobsen constraint, where B is forced to be at the ideal distance of A.
///
/// As described in [Robert Bodea's rope simulation article](https://owlree.blog/posts/simulating-a-rope.html).
pub fn jakobsen_constraint_single(a: Vec3, b: Vec3, ideal_distance: f32) -> Vec3 {
    a + (b - a).normalize() * ideal_distance
}

/// Describes the current simulation state of a rope point.
#[derive(Clone, Copy)]
pub struct RopeTensionData {
    /// Point index of the previous rope bind. This is, at most, the current point index minus 1.
    pub previous_bind_index: usize,
    /// Point index of the next rope bind. This is, at least, the current point index.
    pub next_bind_index: usize,
    /// Normalized vector describing the direction of tension here in the rope, from the beggining of the rope toward the end.
    pub tension_direction: Vec3,
    /// Actual distance between two points of the given section of rope.
    pub section_distance: f32,
    /// Maximum distance between two points of the given section of rope, based on ideal rope length.
    pub max_section_distance: f32,
}

impl Default for RopeTensionData {
    fn default() -> Self {
        Self {
            previous_bind_index: 0,
            next_bind_index: 0,
            tension_direction: Vec3::NEG_Z,
            section_distance: 1.0,
            max_section_distance: 1.0,
        }
    }
}

/// Data for managing a simulated rope.
///
/// I use techniques described in [Robert Bodea's rope simulation article](https://owlree.blog/posts/simulating-a-rope.html).
#[derive(Clone)]
pub struct RopeData {
    /// Number of points in the rope.
    pub point_count: u32,
    /// Ideal distance between points in the rope.
    pub distance_between_points: f32,
    /// Spring constant of the rope.
    pub spring_constant: f32,
    /// Constant acceleration applied to the rope.
    pub acceleration: Vec3,
    /// Number of Jakobsen constraint steps to perform.
    pub constraint_iterations: u32,

    /// All current rope positions.
    pub points: Vec<Vec3>,

    /// All previous rope positions.
    pub previous_points: Vec<Vec3>,

    /// Last computed tension data for each point on the rope.
    pub tension: Vec<RopeTensionData>,
}

impl RopeData {
    /// Generates a new RopeData struct.
    pub fn new(ideal_length: f32, point_distance: f32) -> Self {
        // Find ideal number of points in rope
        let count = (ideal_length / point_distance).round() as u32;

        // Create a line of points along the forward axis
        let mut points: Vec<Vec3> = Vec::with_capacity(count as usize);
        let mut tension: Vec<RopeTensionData> = Vec::with_capacity(count as usize);

        for i in 0..count {
            points.push((i as f32 / count as f32) * Vec3::NEG_Z);
            tension.push(RopeTensionData::default());
        }

        Self {
            point_count: count,
            distance_between_points: ideal_length / (count as f32),
            spring_constant: 5000.0,
            acceleration: vec3(0.0, -9.81, 0.0),
            constraint_iterations: 50,

            points: points.clone(),
            previous_points: points,
            tension,
        }
    }

    /// Returns the point index for the given binding location (between 0 and 1).
    pub fn bind_index(&self, param: f32) -> usize {
        (param.clamp(0.0, 1.0) * ((self.points.len() - 1) as f32)).round() as usize
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

    /// Computes the rope tension system.
    pub fn tension(&mut self, bindings: HashMap<i64, Vec4>) {
        // First, figure out what bindings there are
        let mut bind_indices: Vec<usize> = Vec::with_capacity(bindings.len());

        for (_, b) in bindings.iter() {
            let bind_idx = self.bind_index(b.w);

            // Only add unique bind indices
            if !bind_indices.contains(&bind_idx) {
                bind_indices.push(bind_idx);
            }
        }

        // Sort for consistency
        bind_indices.sort();
    }

    /// Constrains the system X many times, snapping the system back to bound points.
    /// Uses the Jakobsen Method.
    ///
    /// TODO: should this fill a new set of points each iteration instead of operating on the same dataset?
    pub fn constrain(&mut self, bindings: HashMap<i64, Vec4>) {
        // Store binding positions for future reference
        let mut bind_indices: Vec<usize> = Vec::with_capacity(self.points.len());

        // Enforce binding positions, if any are present
        for (_, b) in bindings.iter() {
            let bind_idx = self.bind_index(b.w);
            bind_indices.push(bind_idx);

            self.points[bind_idx] = b.xyz();
        }

        // Run many iterations
        for _ in 0..self.constraint_iterations {
            // Force points towards/away from each other to meet the constraint
            for idx in 1..self.points.len() {
                let lock_a = bind_indices.contains(&idx);
                let lock_b = bind_indices.contains(&(idx - 1));

                if lock_a && lock_b {
                    continue;
                }
                if lock_a {
                    self.points[idx - 1] = jakobsen_constraint_single(
                        self.points[idx],
                        self.points[idx - 1],
                        self.distance_between_points,
                    );
                    continue;
                }
                if lock_b {
                    self.points[idx] = jakobsen_constraint_single(
                        self.points[idx - 1],
                        self.points[idx],
                        self.distance_between_points,
                    );
                    continue;
                }

                // Constrain with previous point
                (self.points[idx], self.points[idx - 1]) = jakobsen_constraint(
                    self.points[idx],
                    self.points[idx - 1],
                    self.distance_between_points,
                );
            }
        }
    }
}

impl Default for RopeData {
    fn default() -> Self {
        Self::new(1.0, 0.1)
    }
}
