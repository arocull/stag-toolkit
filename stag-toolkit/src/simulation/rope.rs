use std::collections::HashMap;

use glam::{FloatExt, Vec3, Vec4, Vec4Swizzles, vec3};

/// Returns a tuple of values A and B, constrainted within the given distance from each other.
/// Acts as a double-sided Jakobsen constraint, with added strain.
pub fn jakobsen_constraint(a: Vec3, b: Vec3, ideal_distance: f32) -> (Vec3, Vec3) {
    let (o, d) = (a - b).normalize_and_length();
    // Half the change in distance we need in order to meet the constraint.
    let distance_offset = (d - ideal_distance) * 0.5;
    // Direction from previous point to this one, multiplied by offset distance.
    let offset = o * distance_offset;
    (a - offset, b + offset)
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
/// I use techniques described in [Robert Badea's rope simulation article](https://owlree.blog/posts/simulating-a-rope.html).
#[derive(Clone)]
pub struct RopeData {
    /// Number of points in the rope.
    pub point_count: usize,
    /// Ideal distance between points in the rope.
    pub distance_between_points: f32,
    /// Spring constant of the rope.
    pub spring_constant: f32,
    /// Constant acceleration applied to the rope.
    pub acceleration: Vec3,
    /// Number of Jakobsen constraint steps to perform.
    pub constraint_iterations: u32,

    /// All current simulated rope positions, with tension.
    pub points: Vec<Vec3>,

    /// All previous simulated rope positions.
    pub points_simulated_previous: Vec<Vec3>,

    /// All rope positions that are pinned via bindings.
    /// Used to optimize constraint iterations.
    pinned: Vec<bool>,

    /// Last computed tension data for each point on the rope.
    tension: Vec<RopeTensionData>,
}

impl RopeData {
    /// Generates a new RopeData struct.
    pub fn new(ideal_length: f32, point_distance: f32) -> Self {
        // Find ideal number of points in rope
        let count = ((ideal_length / point_distance).round() as usize).max(2);

        // Create a line of points along the forward axis
        let mut points: Vec<Vec3> = Vec::with_capacity(count);
        for i in 0..count {
            points.push((i as f32 / count as f32) * Vec3::NEG_Z);
        }

        Self {
            point_count: count,
            distance_between_points: ideal_length / (count as f32),
            spring_constant: 5000.0,
            acceleration: vec3(0.0, -9.81, 0.0),
            constraint_iterations: 50,

            points: points.clone(),
            points_simulated_previous: points,
            pinned: vec![false; count],
            tension: vec![RopeTensionData::default(); count],
        }
    }

    /// Returns the point index for the given binding location (between 0 and 1).
    /// Assumes passed "param" value is between 0 and 1.
    pub fn bind_index(&self, param: f32) -> usize {
        (param * ((self.point_count - 1) as f32)).round() as usize
    }

    /// Returns the rope factor based off the given bind index.
    pub fn bind_factor(&self, index: usize) -> f32 {
        index as f32 / (self.point_count - 1) as f32
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
    pub fn step(&mut self, delta_time: f64) {
        // let delta_time_squared: f32 = (delta_time * delta_time) as f32;
        let accel = self.acceleration * ((delta_time * delta_time) as f32);
        for (idx, point) in self.points.iter_mut().enumerate() {
            // Perform a Verlet integration of the given point
            let p = *point;
            *point = (p * 2.0) - self.points_simulated_previous[idx] + accel;
            self.points_simulated_previous[idx] = p;
        }
    }

    /// Converts a keyed-by-ID bindings map to a keyed-by-index map of unique bindings.
    pub fn unique_bind_map(&self, bindings: &HashMap<i64, Vec4>) -> HashMap<usize, Vec3> {
        let mut unique: HashMap<usize, Vec3> = HashMap::with_capacity(bindings.len());

        for b in bindings.values() {
            unique.insert(self.bind_index(b.w), b.xyz());
        }

        unique
    }

    /// Returns the immediate indices of the binds smaller and greater than the given index, if present.
    pub fn get_surrounding_bind_indices<T>(
        &self,
        idx: usize,
        binding_map: &HashMap<usize, T>,
    ) -> (usize, bool, usize, bool) {
        // Figure out binding indices bounding this section
        let mut next_smallest: usize = 0;
        let mut next_largest: usize = self.points.len() - 1;
        let mut has_smallest: bool = false;
        let mut has_largest: bool = false;

        // First, find smallest index
        for (bind_idx, _) in binding_map.iter() {
            if *bind_idx < idx && *bind_idx >= next_smallest {
                has_smallest = true;
                next_smallest = *bind_idx;
            }
        }

        // Then, find largest index, ensuring it's larger than the smallest
        for (bind_idx, _) in binding_map.iter() {
            if *bind_idx > idx && *bind_idx <= next_largest && *bind_idx > next_smallest {
                has_largest = true;
                next_largest = *bind_idx;
            }
        }

        (next_smallest, has_smallest, next_largest, has_largest)
    }

    /// Recomputes the rope tension system.
    pub fn tension(&mut self, binding_map: &HashMap<usize, Vec3>) {
        // First find first and last bind indices
        for idx in 0..self.tension.len() {
            // Figure out binding indices bounding this section
            let (next_smallest, has_smallest, next_largest, has_largest) =
                self.get_surrounding_bind_indices(idx, binding_map);

            // Find actual point positions
            let point_previous: Vec3;
            if !has_smallest {
                point_previous = self.points[idx];
            } else if let Some(prev) = binding_map.get(&next_smallest) {
                point_previous = *prev;
            } else {
                point_previous = self.points[next_smallest];
            }

            let point_next: Vec3;
            if !has_largest {
                point_next = self.points[idx];
            } else if let Some(next) = binding_map.get(&next_largest) {
                point_next = *next;
            } else {
                point_next = self.points[next_largest];
            }

            let tension_direction: Vec3 = (point_next - point_previous).normalize_or_zero();

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
    pub fn constrain(&mut self, binding_map: &HashMap<usize, Vec3>) {
        // Figure out which points are pinned by the hash map so we only have to find them once
        for (idx, val) in self.pinned.iter_mut().enumerate() {
            *val = binding_map.contains_key(&idx);
        }

        // Run many iterations
        for _ in 0..self.constraint_iterations {
            // Force points towards/away from each other to meet the constraint.
            // Don't move points that are pinned down.
            for (idx, pinned) in self.pinned.iter().enumerate().skip(1) {
                let previdx = idx - 1;

                if *pinned {
                    if self.pinned[previdx] {
                        continue;
                    }

                    self.points[previdx] = jakobsen_constraint_single(
                        self.points[idx],
                        self.points[previdx],
                        self.distance_between_points,
                    );
                    continue;
                }
                if self.pinned[previdx] {
                    self.points[idx] = jakobsen_constraint_single(
                        self.points[previdx],
                        self.points[idx],
                        self.distance_between_points,
                    );
                    continue;
                }

                // Constrain with previous point
                (self.points[idx], self.points[previdx]) = jakobsen_constraint(
                    self.points[idx],
                    self.points[previdx],
                    self.distance_between_points,
                );
            }

            // Enforce binding positions, if any are present
            for (idx, b) in binding_map.iter() {
                self.points[*idx] = *b;
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

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{math::delta::assert_in_delta, simulation::rope::jakobsen_constraint};

    use super::RopeData;

    #[test]
    fn binds_and_factor_conversion() {
        let rope = RopeData::new(10.0, 0.1);
        assert_eq!(0.0, rope.bind_factor(0));
        assert_eq!(1.0, rope.bind_factor(99));

        assert_eq!(0, rope.bind_index(0.0));
        assert_eq!(99, rope.bind_index(1.0));

        assert_eq!(50, rope.bind_index(0.5050505));
        assert_eq!(0.5050505, rope.bind_factor(50));

        let settings = vec![(10.0, 0.1), (25.0, 0.2), (45.7, 0.05)];
        for s in settings {
            // Rope with 100 points
            let rope = RopeData::new(s.0, s.1);

            for i in 0..rope.points.len() {
                let factor = rope.bind_factor(i);
                let idx = rope.bind_index(factor);

                assert_eq!(
                    i, idx,
                    "iteration {0} with length {1} and distance {2}",
                    i, s.0, s.1
                );
            }
        }
    }

    #[test]
    fn test_jakobsen_constraint() {
        let test_cases = [
            (Vec3::splat(2.0), Vec3::splat(-2.0), 0.5),
            (Vec3::splat(2.0), Vec3::splat(-2.0), 1.0),
            (Vec3::splat(1.0), Vec3::splat(2.0), 35.7),
        ];

        const EPSILON: f32 = 1e-6;

        for (idx, tcase) in test_cases.iter().enumerate() {
            let (l, r) = jakobsen_constraint(tcase.0, tcase.1, tcase.2);

            let middle = (tcase.0 + tcase.1) * 0.5;
            let dtl = (tcase.0 - middle).normalize_or_zero();
            let dtr = (tcase.1 - middle).normalize_or_zero();
            let dl = (l - middle).normalize_or_zero();
            let dr = (r - middle).normalize_or_zero();

            assert_in_delta(
                tcase.2,
                l.distance(r),
                EPSILON,
                format!("case {idx}: distance was not the same"),
            );
            assert_in_delta(
                0.0,
                dl.distance(dtl),
                EPSILON,
                format!(
                    "case {idx}: constrained {l} not in same direction as original {0}\t{dl} != {dtl}",
                    tcase.0
                ),
            );
            assert_in_delta(
                0.0,
                dr.distance(dtr),
                EPSILON,
                format!(
                    "case {idx}: constrained {r} not in same direction as original {0}\t{dr} != {dtr}",
                    tcase.1
                ),
            );
        }
    }
}
