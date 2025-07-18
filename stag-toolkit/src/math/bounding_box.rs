use glam::Vec3;

/// An axis-aligned Bounding Box.
/// Useful for managing volume bounds and intersections.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct BoundingBox {
    /// Minimum axis values of the bounding box.
    pub minimum: Vec3,
    /// Maximum axis values of the bounding box.
    pub maximum: Vec3,
}

impl BoundingBox {
    pub fn new(minimum: Vec3, maximum: Vec3) -> Self {
        Self { minimum, maximum }
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
    pub fn add_margin(&self, margin: f32) -> Self {
        Self {
            minimum: self.minimum - Vec3::splat(margin),
            maximum: self.maximum + Vec3::splat(margin),
        }
    }

    /// Returns a new bounding box with a vector to expand boundaries by.
    pub fn add_vector(&self, v: Vec3) -> Self {
        Self {
            minimum: self.minimum - v,
            maximum: self.maximum + v,
        }
    }

    /// Returns the enclosed volume of bounding box.
    pub fn volume(&self) -> f32 {
        let dim = self.size();
        dim.x * dim.y * dim.z
    }
}

#[cfg(test)]
mod tests {
    use crate::math::bounding_box::BoundingBox;
    use glam::Vec3;

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
    fn test_add_margin() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        let added = aabb.add_margin(1.0);
        assert_eq!(added.minimum, Vec3::NEG_ONE);
        assert_eq!(added.maximum, Vec3::ONE);
    }

    #[test]
    fn test_add_vector() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        let added = aabb.add_vector(Vec3::new(1.0, 0.5, 1.0));

        assert_eq!(added.minimum, Vec3::new(-1.0, -0.5, -1.0));
        assert_eq!(added.maximum, Vec3::new(1.0, 0.5, 1.0));
    }

    #[test]
    fn volume() {
        let aabb = BoundingBox::new(Vec3::ZERO, Vec3::ONE);
        assert_eq!(aabb.volume(), 1.0);

        let aabb = BoundingBox::new(Vec3::NEG_ONE, Vec3::ONE);
        assert_eq!(aabb.volume(), 8.0);
    }
}
