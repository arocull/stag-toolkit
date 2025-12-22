use crate::math::{bounding_box::BoundingBox, projection::Plane};
use glam::{Vec3, Vec4};

/// A set of points for analysis.
pub trait PointCloud {
    /// Returns the axis-aligned bounding box for the given list of points.
    fn bounds(&self) -> BoundingBox;

    /// Returns the indices two most distant pairing of points in the cloud, given the bounding box.
    fn distant(&self, aabb: BoundingBox) -> (usize, usize);

    /// Returns the index of the most distant point from the given line.
    fn distant_line(&self, from: Vec3, to: Vec3) -> usize;

    /// Returns the index of the most distant point from the given plane.
    fn distant_plane(&self, from: Vec4) -> usize;
}

impl PointCloud for Vec<Vec3> {
    fn bounds(&self) -> BoundingBox {
        // If we have no points, returns an empty bounding box.
        if self.is_empty() {
            return BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        }

        // Otherwise, start bounding box on first item.
        let mut aabb = BoundingBox::new(self[0], Vec3::ZERO);

        // Expand AABB to contain each item.
        for item in self.iter() {
            aabb = aabb.enclose(*item);
        }

        aabb
    }

    fn distant(&self, aabb: BoundingBox) -> (usize, usize) {
        if !aabb.zero() {
            let axis = aabb.size().max_position();
            let mut min_idx = 0;
            let mut max_idx = 0;
            let mut max: f32 = 0.0;
            let mut min: f32 = 0.0;

            for (idx, pt) in self.iter().enumerate() {
                let d = (*pt)[axis];

                if idx == 0 || d < min {
                    min_idx = idx;
                    min = d;
                }

                if idx == 0 || d > max {
                    max_idx = idx;
                    max = d;
                }
            }

            return (min_idx, max_idx);
        }
        (0, 0)
    }

    fn distant_line(&self, from: Vec3, to: Vec3) -> usize {
        // These points are the same!
        if from.abs_diff_eq(to, 1e-6) {
            return 0;
        }

        let mut max: f32 = 0.0;
        let mut i: usize = 0;
        let relative = (from - to).normalize();

        for (idx, pt) in self.iter().enumerate() {
            // Cross product between our line direction and the direction from the segment start to the point,
            // then cross it with the line direction again to isolate the axis
            let cc = relative.cross(from - *pt).cross(relative);
            // Skip points that fall directly on our line
            if cc.abs_diff_eq(Vec3::ZERO, 1e-6) {
                continue;
            }

            let normal = cc.normalize();
            let d: f32 = (normal.dot(from) - normal.dot(*pt)).abs();

            if d > max {
                max = d;
                i = idx;
            }
        }

        i
    }

    fn distant_plane(&self, from: Vec4) -> usize {
        let mut max: f32 = 0.0;
        let mut i: usize = 0;

        for (idx, pt) in self.iter().enumerate() {
            let d = from.signed_distance(*pt).abs();

            if d > max {
                max = d;
                i = idx;
            }
        }

        i
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        math::{delta::assert_in_delta_vector, projection::plane},
        mesh::trimesh::{Triangle, TriangleOperations},
    };

    use super::*;

    #[test]
    fn bounds() {
        let pts = vec![
            Vec3::new(0.0, 2.0, 2.0),
            Vec3::new(-2.0, 2.0, 2.0),
            Vec3::new(3.0, 2.0, 5.0),
            Vec3::new(3.0, 4.0, 0.0),
        ];

        let aabb = pts.bounds();

        assert_in_delta_vector(
            Vec3::new(-2.0, 2.0, 0.0),
            aabb.minimum,
            1e-6,
            "Bounds start in proper position",
        );

        assert_in_delta_vector(
            Vec3::new(5.0, 2.0, 5.0),
            aabb.size(),
            1e-6,
            "Bounds have correct size",
        );
    }

    #[test]
    fn distant() {
        let pts = vec![
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(-3.0, 7.0, 2.0),
            Vec3::new(-6.0, 7.0, 15.0),
            Vec3::new(-10.0, 0.0, 0.0),
        ];

        let aabb = pts.bounds();
        let (dist_min, dist_max) = pts.distant(aabb);

        assert_eq!(
            dist_min, 3,
            "minimum point index should be 3, got {dist_min}"
        );
        assert_eq!(
            dist_max, 0,
            "maximum point index should be 0, got {dist_max}"
        );

        // Test distance to line
        let furthest_from_line = pts.distant_line(pts[dist_min], pts[dist_max]);
        assert_eq!(
            furthest_from_line, 2,
            "furthest point from line should be 2, got {furthest_from_line}"
        );

        // Test distance to plane
        let tri: Triangle = [dist_min, dist_max, furthest_from_line];
        let p = plane(pts[dist_min], tri.normal(&pts));

        let furthest_from_plane = pts.distant_plane(p);
        assert_eq!(
            furthest_from_plane, 1,
            "furthest point from plane should be 1, got {furthest_from_plane}"
        );
    }
}
