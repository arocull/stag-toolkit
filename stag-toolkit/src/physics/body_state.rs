use glam::{Mat4, Vec3};

/// Modifiable state of a physics body.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct BodyState {
    /// Global space orientation of the [PhysicsBody].
    pub transform: Mat4,
    /// Linear velocity of the [PhysicsBody].
    pub linear_velocity: Vec3,
    /// Angular velocity of the [PhysicsBody].
    pub angular_velocity: Vec3,
}

impl BodyState {
    fn new(transform: Mat4) -> Self {
        Self {
            transform,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }
}

impl Default for BodyState {
    fn default() -> Self {
        Self::new(Mat4::IDENTITY)
    }
}
