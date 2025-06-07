use crate::animation::mixable::Mixable;
use glam::{Quat, Vec3};
use std::collections::HashMap;

/// Simple identifier for a pose channel.
pub type PoseChannel = u64;

/// A 3D animation pose.
#[derive(Clone)]
pub struct Pose {
    blendshapes: HashMap<PoseChannel, f32>,
    positions: HashMap<PoseChannel, Vec3>,
    rotations: HashMap<PoseChannel, Quat>,
    scales: HashMap<PoseChannel, Vec3>,
}

impl Pose {
    /// Linearly interpolates the blendshapes of the pose.
    /// If a channel does not exist in one pose or the other, uses whatever existing value there is.
    pub fn interpolate(&mut self, rhs: &Self, blend: f32) {
        self.blendshapes.interpolate(&rhs.blendshapes, blend);
        self.positions.interpolate(&rhs.positions, blend);
        self.rotations.interpolate(&rhs.rotations, blend);
        self.scales.interpolate(&rhs.scales, blend);
    }

    /// Adds the right-hand side blendshape keys to the left-hand side ones.
    pub fn add(&mut self, rhs: &Self, weight: f32) {
        self.blendshapes.add(&rhs.blendshapes, weight);
        self.positions.add(&rhs.positions, weight);
        self.rotations.add(&rhs.rotations, weight);
        self.scales.add(&rhs.scales, weight);
    }

    /// Multiplies the left-hand side blendshape keys by the right-hand side blendshape keys.
    /// TODO: probably want these to be multiplied individually, not as a whole function?
    pub fn multiply(&mut self, rhs: &Self) {
        self.blendshapes.multiply(&rhs.blendshapes);
        self.positions.multiply(&rhs.positions);
        self.rotations.multiply(&rhs.rotations);
        self.scales.multiply(&rhs.scales);
    }
}
