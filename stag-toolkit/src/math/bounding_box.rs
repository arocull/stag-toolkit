use glam::{Mat4, Vec3, Vec4Swizzles};
use std::ops::Mul;

/// An axis-aligned Bounding Box.
/// Useful for managing volume bounds and intersections.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct BoundingBox {
    /// Minimum axis values of the bounding box. A corner point.
    pub minimum: Vec3,
    /// Maximum axis values of the bounding box. A corner point.
    pub maximum: Vec3,
}

impl BoundingBox {
    /// Returns a new bounding box.
    /// Every axis of the `minimum` vector should be less than or equal to the corresponding `maximum` axis.
    pub fn new(minimum: Vec3, maximum: Vec3) -> Self {
        Self { minimum, maximum }
    }

    /// Creates a bounding box that encloses the given list of points.
    pub fn from(points: &[Vec3]) -> Self {
        if points.is_empty() {
            return Self::default();
        }

        let mut b = Self::new(points[0], points[0]);
        for pt in points.iter() {
            b = b.enclose(*pt);
        }
        b
    }

    /// Returns the central position of the Bounding Box.
    pub fn center(&self) -> Vec3 {
        self.minimum.midpoint(self.maximum)
    }

    /// Returns the size of the bounding box.
    pub fn size(&self) -> Vec3 {
        self.maximum - self.minimum
    }

    /// Returns a new bounding box with proper minimum and maximum extents enforced.
    pub fn abs(&self) -> Self {
        Self {
            minimum: self.minimum.min(self.maximum),
            maximum: self.maximum.max(self.minimum),
        }
    }

    /// Returns a new bounding box with a flat scalar to expand all boundaries by.
    pub fn expand_margin(&self, margin: f32) -> Self {
        Self {
            minimum: self.minimum - Vec3::splat(margin),
            maximum: self.maximum + Vec3::splat(margin),
        }
    }

    /// Returns a new bounding box with a vector to expand boundaries by.
    pub fn expand_vector(&self, v: Vec3) -> Self {
        Self {
            minimum: self.minimum - v,
            maximum: self.maximum + v,
        }
    }

    /// Returns a new bounding box translated by the given vector.
    pub fn translate(&self, v: Vec3) -> Self {
        Self {
            minimum: self.minimum + v,
            maximum: self.maximum + v,
        }
    }

    /// Returns the enclosed volume of bounding box.
    pub fn volume(&self) -> f32 {
        let dim = self.size();
        dim.x * dim.y * dim.z
    }

    /// Returns a new bounding box that encloses the given bounding boxes.
    pub fn join(&self, other: &Self) -> Self {
        Self {
            minimum: self.minimum.min(other.minimum),
            maximum: self.maximum.max(other.maximum),
        }
    }

    /// Returns a new bounding box which encloses the given point.
    pub fn enclose(&self, point: Vec3) -> Self {
        Self {
            minimum: self.minimum.min(point),
            maximum: self.maximum.max(point),
        }
    }

    /// Returns true if the bounding box has no volume.
    pub fn zero(&self) -> bool {
        self.minimum.eq(&self.maximum)
    }
}

impl Mul<BoundingBox> for Mat4 {
    type Output = BoundingBox;

    fn mul(self, rhs: BoundingBox) -> Self::Output {
        // https://web.archive.org/web/20220317024830/https://dev.theomader.com/transform-bounding-boxes/
        let xa = self.col(0) * rhs.minimum.x;
        let xb = self.col(0) * rhs.maximum.x;
        let ya = self.col(1) * rhs.minimum.y;
        let yb = self.col(1) * rhs.maximum.y;
        let za = self.col(2) * rhs.minimum.z;
        let zb = self.col(2) * rhs.maximum.z;

        let min_bound = (xa.min(xb) + ya.min(yb) + za.min(zb)).xyz();
        let max_bound = (xa.max(xb) + ya.max(yb) + za.max(zb)).xyz();

        let translation = self.w_axis.xyz();

        Self::Output {
            minimum: min_bound + translation,
            maximum: max_bound + translation,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::math::bounding_box::BoundingBox;
    use glam::{Mat4, Vec3};

    #[test]
    fn test_center() {
        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        assert_eq!(aabb.center(), Vec3::ZERO);

        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.center(), Vec3::splat(0.5));

        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::NEG_ONE);
        assert_eq!(aabb.center(), Vec3::splat(-0.5));
    }

    #[test]
    fn test_size() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.size(), Vec3::ONE);

        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        assert_eq!(aabb.size(), Vec3::splat(2.0));
    }

    #[test]
    fn test_abs() {
        let aabb_proper = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        let aabb_inverted = BoundingBox::new(Vec3::ONE, Vec3::NEG_ONE);

        assert_eq!(aabb_inverted.abs(), aabb_proper);
    }

    #[test]
    fn test_expand_margin() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        let added = aabb.expand_margin(1.0);
        assert_eq!(added.minimum, Vec3::NEG_ONE);
        assert_eq!(added.maximum, Vec3::ONE);
    }

    #[test]
    fn test_expand_vector() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        let added = aabb.expand_vector(Vec3::new(1.0, 0.5, 1.0));

        assert_eq!(added.minimum, Vec3::new(-1.0, -0.5, -1.0));
        assert_eq!(added.maximum, Vec3::new(1.0, 0.5, 1.0));
    }

    #[test]
    fn test_translate() {
        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        let translated = aabb.translate(Vec3::ONE);
        assert_eq!(translated.minimum, Vec3::ZERO);
        assert_eq!(translated.maximum, Vec3::splat(2.0));
    }

    #[test]
    fn test_volume() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.volume(), 1.0);

        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        assert_eq!(aabb.volume(), 8.0);
    }

    #[test]
    fn test_join() {
        let a = BoundingBox::new(Vec3::ZERO, Vec3::ONE);
        let b = BoundingBox::new(Vec3::NEG_ONE, Vec3::ZERO);
        let joined = a.join(&b);

        assert_eq!(joined.minimum, Vec3::NEG_ONE);
        assert_eq!(joined.maximum, Vec3::ONE);
        assert_eq!(joined.volume(), 8.0, "volume increased");
    }

    #[test]
    fn test_transform() {
        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);

        let scale = Mat4::from_scale(Vec3::splat(2.0));
        let scaled = BoundingBox::new(Vec3::splat(-2.0), Vec3::splat(2.0));
        assert_eq!(scale * aabb, scaled, "scaled by 2");

        let translate = Mat4::from_translation(Vec3::ONE);
        let translated = BoundingBox::new(Vec3::ZERO, Vec3::splat(2.0));
        assert_eq!(translate * aabb, translated, "translated by (1, 1, 1)");

        let rotate_90 = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let rotate_90n = Mat4::from_rotation_y(-std::f32::consts::FRAC_PI_2);
        let rotated_90 = aabb;
        assert_eq!(rotate_90 * aabb, rotated_90, "rotating 90 degrees");
        assert_eq!(rotate_90n * aabb, rotated_90, "rotating -90 degrees");

        let rotate_45 = Mat4::from_rotation_y(std::f32::consts::FRAC_PI_4);
        let rotate_45n = Mat4::from_rotation_y(-std::f32::consts::FRAC_PI_4);
        let rotated_45 = BoundingBox::new(
            Vec3::new(-std::f32::consts::SQRT_2, -1.0, -std::f32::consts::SQRT_2),
            Vec3::new(std::f32::consts::SQRT_2, 1.0, std::f32::consts::SQRT_2),
        );
        assert_eq!(rotate_45 * aabb, rotated_45, "rotating 45 degrees");
        assert_eq!(rotate_45n * aabb, rotated_45, "rotating -45 degrees");
    }
}
