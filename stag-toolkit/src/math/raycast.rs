use core::fmt;
use glam::{Mat4, Vec3};
use std::{
    fmt::{Display, Formatter},
    ops::Mul,
};

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

impl Mul<RaycastParameters> for Mat4 {
    type Output = RaycastParameters;

    /// Returns a new set of raycast parameters transformed by the given matrix.
    fn mul(self, rhs: RaycastParameters) -> Self::Output {
        RaycastParameters::new(
            self.transform_point3(rhs.origin),
            self.transform_vector3(rhs.direction),
            rhs.max_depth,
            rhs.hit_backfaces,
        )
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

impl Display for RaycastParameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ origin: {0}, direction: {1}, max_depth: {2}, hit_backfaces: {3} }}",
            self.origin, self.direction, self.max_depth, self.hit_backfaces
        )
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

impl Mul<RaycastResult> for Mat4 {
    type Output = RaycastResult;

    /// Returns a new set of raycast parameters transformed by the given matrix.
    fn mul(self, rhs: RaycastResult) -> Self::Output {
        let mut result = rhs;
        result.point = self.transform_point3(result.point);
        result.normal = self.transform_vector3(result.normal);
        result
    }
}

impl Display for RaycastResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ point: {0}, normal: {1}, depth: {2} }}",
            self.point, self.normal, self.depth
        )
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
