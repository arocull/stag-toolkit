use glam::Vec3;

/// Features for raycasting objects.
pub trait Raycast {
    /// Perform a singular raycast on the object from the given point to the end point.
    /// `max_depth` is the maximum depth a collision can occur at.
    /// If `backfaces` is true, the direction of the face is ignored.
    ///
    /// The raycast result for the shallowest collision point is returned.
    /// Returns [None] if the ray did not hit.
    fn raycast(&self, parameters: RaycastParameters) -> Option<RaycastResult>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RaycastParameters {
    pub origin: Vec3,
    pub direction: Vec3,
    pub max_depth: f32,
    pub hit_backfaces: bool,
}

impl RaycastParameters {
    pub fn new(origin: Vec3, direction: Vec3, max_depth: f32, hit_backfaces: bool) -> Self {
        Self {
            origin,
            direction,
            max_depth,
            hit_backfaces,
        }
    }
}

impl Default for RaycastParameters {
    fn default() -> Self {
        Self {
            origin: Vec3::ZERO,
            direction: Vec3::Z,
            max_depth: f32::INFINITY,
            hit_backfaces: false,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
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

pub trait RaycastResultReducer {
    fn nearest(&self) -> Option<RaycastResult>;
}

impl RaycastResultReducer for Vec<Option<RaycastResult>> {
    fn nearest(&self) -> Option<RaycastResult> {
        let result = self.iter().reduce(|lhs, rhs| {
            if let Some(left) = lhs {
                if let Some(right) = rhs {
                    if left.depth <= right.depth {
                        return lhs;
                    }
                    return rhs;
                }
                return lhs;
            }
            rhs
        });

        if let Some(result) = result {
            return *result;
        }
        None
    }
}

impl RaycastResultReducer for Vec<RaycastResult> {
    fn nearest(&self) -> Option<RaycastResult> {
        let result = self.iter().reduce(|lhs, rhs| {
            if lhs.depth <= rhs.depth {
                return lhs;
            }
            rhs
        });

        if let Some(result) = result {
            return Some(*result);
        }
        None
    }
}
