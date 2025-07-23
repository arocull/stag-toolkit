use glam::Vec3;

#[derive(Copy, Clone, Default, Debug)]
pub struct RaycastResult {
    /// Point where ray intersected with the collision.
    pub point: Vec3,
    /// Surface normal of collision.
    pub normal: Vec3,
    /// How far from the ray origin the collision hit.
    pub depth: f32,

    /// Optional face index of collision point.
    pub face_index: Option<usize>,
    /// Optional barycentric coordinate of a face.
    pub barycentric: Option<Vec3>,
}

impl RaycastResult {
    /// Returns this [RaycastResult], or [None] if the result depth is too large.
    pub fn max_depth(&self, depth: f32) -> Option<Self> {
        if self.depth > depth {
            return None;
        }
        Some(*self)
    }
}

/// Features for raycasting objects.
pub trait Raycast {
    /// Perform a singular raycast on the object from the given point to the end point.
    /// `max_depth` is the maximum depth a collision can occur at.
    /// If `backfaces` is true, the direction of the face is ignored.
    ///
    /// The raycast result for the shallowest collision point is returned.
    /// Returns [None] if the ray did not hit.
    fn raycast(
        &self,
        origin: Vec3,
        dir: Vec3,
        max_depth: f32,
        backfaces: bool,
    ) -> Option<RaycastResult>;
}
