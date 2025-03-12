use std::collections::HashMap;

use glam::{vec3, FloatExt, Vec3, Vec4, Vec4Swizzles};

/// Returns a tuple of values A and B, constrainted within the given distance from each other.
/// Acts as a double-sided Jakobsen constraint, with added strain.
pub fn jakobsen_constraint(a: Vec3, b: Vec3, ideal_distance: f32) -> (Vec3, Vec3) {
    // Half the change in distance we need in order to meet the constraint.
    let distance_offset = (a.distance(b) - ideal_distance) * 0.5;
    // Direction from previous point to this one, multiplied by offset distance.
    let offset = (b - a).normalize() * distance_offset;
    (a + offset, b - offset)
}

/// Returns a one-sided Jakobsen constraint, with added strain, where B is forced to be at the ideal distance of A.
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
    /// Current index's percentage factor between these two points.
    pub factor: f32,
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
            factor: 0.0,
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

    /// All current rope positions, with tension.
    pub points: Vec<Vec3>,

    /// All current simulated rope positions.
    pub points_simulated: Vec<Vec3>,

    /// All previous simulated rope positions.
    pub points_simulated_previous: Vec<Vec3>,

    /// Last computed tension data for each point on the rope.
    pub tension: Vec<RopeTensionData>,
}

impl RopeData {
    /// Generates a new RopeData struct.
    pub fn new(ideal_length: f32, point_distance: f32) -> Self {
        // Find ideal number of points in rope
        let count = ((ideal_length / point_distance).round() as u32).max(2);

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
            points_simulated: points.clone(),
            points_simulated_previous: points,
            tension,
        }
    }

    /// Returns the point index for the given binding location (between 0 and 1).
    /// Assumes passed "param" value is between 0 and 1.
    pub fn bind_index(&self, param: f32) -> usize {
        (param * ((self.points.len() - 1) as f32)).round() as usize
    }

    /// Returns the calculated slack at the given index.
    /// 1 is fully slack, 0 is fully stretched.
    pub fn slack(&self, index: usize) -> f32 {
        (self.tension[index].section_distance - self.tension[index].max_section_distance)
            .remap(-self.tension[index].max_section_distance, 0.0, 1.0, 0.0)
            .clamp(0.0, 1.0)
        // .powi(2)
    }

    /// Fetches a linearized position based on the bounding binding locations, if possible.
    pub fn fetch_linear_point(
        &self,
        index: usize,
        binding_map: &HashMap<usize, Vec3>,
    ) -> Option<Vec3> {
        if let Some(prev_bind) = binding_map.get(&self.tension[index].previous_bind_index) {
            if let Some(next_bind) = binding_map.get(&self.tension[index].next_bind_index) {
                return Some(prev_bind.lerp(*next_bind, self.tension[index].factor));
            }
        }
        None
    }

    /// Steps the simulation forward by many X seconds using Verlet integration.
    /// Does NOT apply constraints.
    pub fn step(&mut self, delta_time: f64, binding_map: &HashMap<usize, Vec3>) {
        let delta_time_squared: f32 = (delta_time * delta_time) as f32;
        for idx in 0..self.points.len() {
            let slack = self.slack(idx);

            // Perform a basic point simulation
            let p = self.points_simulated[idx];
            let mut simulated_pos = 2.0 * p - self.points_simulated_previous[idx]
                + self.acceleration * delta_time_squared;
            self.points_simulated[idx] = simulated_pos;

            // Automatically interpolate position with a straight line based on our rope slack
            // However, keep this data separate from the simulation
            if let Some(tense_point) = self.fetch_linear_point(idx, binding_map) {
                simulated_pos = simulated_pos.lerp(tense_point, 1.0 - slack);
            }

            self.points[idx] = simulated_pos;
            self.points_simulated_previous[idx] = p;
        }
    }

    /// Converts a keyed-by-ID bindings map to a keyed-by-index map of unique bindings.
    pub fn unique_bind_map(&self, bindings: &HashMap<i64, Vec4>) -> HashMap<usize, Vec3> {
        let mut unique: HashMap<usize, Vec3> = HashMap::new();

        for (_, b) in bindings.iter() {
            unique.insert(self.bind_index(b.w), b.xyz());
        }

        unique
    }

    /// Recomputes the rope tension system.
    pub fn tension(&mut self, binding_map: &HashMap<usize, Vec3>) {
        // First find first and last bind indices
        for idx in 0..self.tension.len() {
            // Figure out binding indices bounding this section
            let mut next_smallest: usize = 0;
            let mut next_largest: usize = self.tension.len() - 1;

            // First, find smallest index
            for (bind_idx, _) in binding_map.iter() {
                if *bind_idx < idx && *bind_idx > next_smallest {
                    next_smallest = *bind_idx;
                }
            }

            // Then, find largest index, ensuring it's larger than the smallest
            for (bind_idx, _) in binding_map.iter() {
                if *bind_idx >= idx && *bind_idx < next_largest && *bind_idx > next_smallest {
                    next_largest = *bind_idx;
                }
            }

            // Find actual point positions
            let point_previous: Vec3;
            if let Some(prev) = binding_map.get(&next_smallest) {
                point_previous = *prev;
            } else {
                point_previous = self.points[next_smallest];
            }

            let point_next: Vec3;
            if let Some(next) = binding_map.get(&next_largest) {
                point_next = *next;
            } else {
                point_next = self.points[next_largest];
            }

            let tension_direction: Vec3 = (point_next - point_previous).normalize();

            let section_distance: f32 = point_previous.distance(point_next);
            let max_section_distance: f32 =
                self.distance_between_points * (next_largest - next_smallest + 1) as f32;

            self.tension[idx] = RopeTensionData {
                previous_bind_index: next_smallest,
                next_bind_index: next_largest,
                factor: (idx - next_smallest) as f32 / (next_largest - next_smallest) as f32,
                tension_direction,
                section_distance,
                max_section_distance,
            }
        }
    }

    /// Constrains the system X many times, snapping the system back to bound points.
    /// Uses the Jakobsen Method.
    ///
    /// TODO: should this fill a new set of points each iteration instead of operating on the same dataset?
    pub fn constrain(&mut self, binding_map: &HashMap<usize, Vec3>) {
        // Enforce binding positions, if any are present
        for (idx, b) in binding_map.iter() {
            self.points[*idx] = *b;
            self.points_simulated[*idx] = *b;
        }

        // Run many iterations
        for _ in 0..self.constraint_iterations {
            // Force points towards/away from each other to meet the constraint
            for idx in 1..self.points.len() {
                let lock_a = binding_map.contains_key(&idx);
                let lock_b = binding_map.contains_key(&(idx - 1));

                if lock_a && lock_b {
                    continue;
                }
                if lock_a {
                    self.points_simulated[idx - 1] = jakobsen_constraint_single(
                        self.points_simulated[idx],
                        self.points_simulated[idx - 1],
                        self.distance_between_points,
                    );
                    continue;
                }
                if lock_b {
                    self.points_simulated[idx] = jakobsen_constraint_single(
                        self.points_simulated[idx - 1],
                        self.points_simulated[idx],
                        self.distance_between_points,
                    );
                    continue;
                }

                // Constrain with previous point
                (self.points_simulated[idx], self.points_simulated[idx - 1]) = jakobsen_constraint(
                    self.points_simulated[idx],
                    self.points_simulated[idx - 1],
                    self.distance_between_points,
                );
            }
        }
    }

    /// Computes the force at the given point.
    pub fn force(&self, index: usize) -> Vec3 {
        let mut left_tension: Vec3 = Vec3::ZERO;
        let mut right_tension: Vec3 = Vec3::ZERO;

        // Determine sampling indices
        if index > 0 {
            // Get left tension
            let idx = index - 1;

            let overstretch = (self.tension[idx].section_distance
                - self.tension[idx].max_section_distance)
                .max(0.0);
            left_tension = -self.tension[idx].tension_direction * overstretch;
        }
        if index < self.points.len() - 1 {
            let idx = index + 1;

            let overstretch = (self.tension[idx].section_distance
                - self.tension[idx].max_section_distance)
                .max(0.0);
            right_tension = self.tension[idx].tension_direction * overstretch;
        }

        (left_tension + right_tension) * self.spring_constant
    }
}

impl Default for RopeData {
    fn default() -> Self {
        Self::new(1.0, 0.1)
    }
}
