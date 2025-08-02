use crate::math::raycast::{RaycastParameters, RaycastResult};
use crate::physics::identity::Identity;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhysicsRaycastParameters {
    pub raycast_parameters: RaycastParameters,
    pub collision_mask: u32,
}

impl PhysicsRaycastParameters {
    pub fn new(raycast_parameters: RaycastParameters, collision_mask: u32) -> Self {
        Self {
            raycast_parameters,
            collision_mask,
        }
    }
}

impl Default for PhysicsRaycastParameters {
    fn default() -> Self {
        Self {
            raycast_parameters: RaycastParameters::default(),
            collision_mask: u32::MAX,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhysicsRaycastResult {
    pub raycast_result: RaycastResult,
    pub body_identifier: Identity,
}
