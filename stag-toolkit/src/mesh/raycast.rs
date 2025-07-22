use glam::{Vec2, Vec3};

pub struct RaycastResult {
    /// Point where ray intersected with the collision.
    pub point: Vec3,
    /// Surface normal of collision.
    pub normal: Vec3,
    /// Whether the ray actually hit the object.
    pub hit: bool,
    /// Whether the hit was back-facing.
    pub backface: bool,
    /// Optional UV coordinate of collision point.
    pub uv: Vec2,
}

pub trait Raycastable {
    /// Perform a raycast on the object from the given point to the end point.
    fn raycast(&self, from: Vec3, to: Vec3, backfaces: bool) -> RaycastResult;
}
