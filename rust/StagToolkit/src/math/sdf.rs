use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

/// Joins two distance functions, using a logarithm for smoothing values.
/// `k = 32.0`` was original suggestion for smoothing value.
pub fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let res = (-k * a).exp() + (-k * b).exp();
    res.max(0.0001).log10() * (-1.0) / k
}

// /// Returns the union of two distance functions.
// pub fn union(a: f32, b: f32) -> f32 {
//     a.max(b)
// }

// /// Returns the intersection of two distance functions.
// pub fn intersection(a: f32, b: f32) -> f32 {
//     a.min(b)
// }

/// Distance function for a sphere.
pub fn sample_sphere(sample_position: Vec3, shape_radius: f32) -> f32 {
    sample_position.length() - shape_radius
}

/// Distance function for a rounded box.
///
/// https://github.com/jasmcole/Blog/blob/master/CSG/src/fragment.ts#L13
/// https://github.com/fogleman/sdf/blob/main/sdf/d3.py#L140
pub fn sample_box_rounded(sample_position: Vec3, shape_dim: Vec3, radius_edge: f32) -> f32 {
    let q = sample_position.abs() - shape_dim * Vec3::splat(0.5) + Vec3::splat(radius_edge);
    let m = q.max(Vec3::ZERO).length();
    m + q.max_element().min(0.0) - radius_edge
}

/// Describes an SDF primitive shape
#[derive(Copy, Clone)]
pub enum ShapeType {
    /// A sphere primitive.
    Sphere,
    /// A rounded box primitive.
    RoundedBox,
}

/// Collection of data describing a Signed Distance Field primitive.
#[derive(Copy, Clone)]
pub struct Shape {
    /// Informs which SDF formula to use when calculating.
    shape: ShapeType,
    /// Describes a sphere or cylinder's radius.
    radius: f32,
    /// Describes the edge rounding on the given shape, if available.
    radius_edge: f32,
    /// Describes the dimensions of a box or cylinder.
    dimensions: Vec3,
    /// Z-Score distance threshold for discarding hull points, when their distance from the shape is over this threshold.
    pub zscore: f32,
    /// Transform of the shape. Applied to position before sampling.
    transform: Mat4,
    /// Inverse transform of the shape. Used for point projection.
    transform_inv: Mat4,
}

impl Shape {
    /// Creates a sphere primitive with the given parameters.
    pub fn sphere(transform: Mat4, radius: f32) -> Self {
        Self {
            shape: ShapeType::Sphere,
            transform,
            transform_inv: transform.inverse(),
            radius,
            radius_edge: 0.0,
            dimensions: Vec3::ZERO,
            zscore: 0.0,
        }
    }
    /// Creates a rounded box primitive with the given parameters.
    pub fn rounded_box(transform: Mat4, dimensions: Vec3, radius_edge: f32) -> Self {
        Self {
            shape: ShapeType::RoundedBox,
            transform,
            transform_inv: transform.inverse(),
            radius: 0.0,
            radius_edge,
            dimensions,
            zscore: 0.0,
        }
    }
    /// Samples the SDF shape at the given point.
    pub fn sample(&self, at: Vec3) -> f32 {
        let position_local = self
            .transform_inv
            .mul_vec4(Vec4::new(at.x, at.y, at.z, 1.0))
            .xyz();
        match self.shape {
            ShapeType::Sphere => sample_sphere(position_local, self.radius),
            ShapeType::RoundedBox => {
                sample_box_rounded(position_local, self.dimensions, self.radius_edge)
            }
        }
    }
    /// Returns the minimum and maximum boundary points of the shape, NOT transformed
    pub fn relative_bounds(&self) -> (Vec3, Vec3) {
        match self.shape {
            ShapeType::Sphere => (Vec3::splat(-self.radius), Vec3::splat(self.radius)),
            ShapeType::RoundedBox => (
                self.dimensions * Vec3::splat(-0.5),
                self.dimensions * Vec3::splat(0.5),
            ),
        }
    }
    /// Returns the transform of the given shape.
    pub fn transform(&self) -> Mat4 {
        self.transform
    }

    /// @TODO
    /// Simplify the given vector of points based on distance to our SDF shape, and a ZScore threshold.
    pub fn simplify_hull(&self, pts: Vec<Vec3>) -> Vec<Vec3> {
        let mut distances = Vec::<f32>::new();
        distances.reserve_exact(pts.len());

        // Calculate mean distance and store distances
        let mut mean: f32 = 0.0;
        for i in 0..pts.len() {
            let dist = self.sample(pts[i]);
            distances.push(dist);
            mean += dist;
        }
        mean /= pts.len() as f32;

        // Calculate standard deviation of the data set
        let mut sdeviation: f32 = 0.0;
        for dist in distances.iter() {
            sdeviation += (dist - mean).powi(2);
        }

        // Allocate new points vector
        let mut new_pts: Vec<Vec3> = Vec::<Vec3>::new();
        new_pts.reserve_exact(pts.len()); // Worst-case scenario, we use all this space

        // Calculate z-score of every point...
        for i in 0..distances.len() {
            let zscore = (distances[i] - mean) / sdeviation;
            // ...and only include point if it falls within our Z-Score threshold
            if zscore < self.zscore {
                new_pts.push(pts[i]);
            }
        }

        new_pts
    }
}

/// Iterates through a shape list, sampling each shape at the given point
/// and smooth unioning the shapes together, returning a distance.
pub fn sample_shape_list(list: &Vec<Shape>, point: Vec3, smoothing_value: f32) -> f32 {
    let mut d: f32 = 1.0;

    for shape in list.iter() {
        d = smooth_union(d, shape.sample(point), smoothing_value)
    }

    d
}

// UNIT TESTS //
#[cfg(test)]
mod tests {
    use glam::Quat;

    use super::*;

    #[test]
    fn test_smooth_union() {
        // Desmos calculator: https://www.desmos.com/calculator/kujnqrwocy
        // A, B, Smoothing (K), expected
        const TEST_MAX_DIFF: f32 = 1e-3f32;
        let cases = [
            (1.0, -1.0, 8.0, -0.43429),
            (1.0, 1.0, 8.0, 0.3966657),
            (-1.0, -1.0, 8.0, -0.471_923_23),
            (-0.1, -0.1, 8.0, -0.081_058_2),
        ];

        for case in cases.iter() {
            let result = smooth_union(case.0, case.1, case.2);
            let diff = (result - case.3).abs();
            assert!(
                diff < TEST_MAX_DIFF,
                "Expected approx {0} but got {1} instead (diff: {2} > {3} ) | a={4}, b={5}, k={6}",
                case.3,
                result,
                diff,
                TEST_MAX_DIFF,
                case.0,
                case.1,
                case.2
            );
        }
    }

    #[test]
    fn sdf_sphere() {
        let sample_points = vec![
            // Test distance while inside sphere
            (Vec3::ZERO, 1.0, -1.0),
            (Vec3::new(0.5, 0.0, 0.0), 1.0, -0.5),
            // ...with larger radius
            (Vec3::ZERO, 2.0, -2.0),
            (Vec3::new(1.0, 0.0, 0.0), 2.0, -1.0),
            // Test distance at surface of sphere
            (Vec3::X, 1.0, 0.0),
            (Vec3::Y, 1.0, 0.0),
            (Vec3::Z, 1.0, 0.0),
            // ...with larger radius
            (Vec3::new(2.0, 0.0, 0.0), 2.0, 0.0),
            (Vec3::new(-2.0, 0.0, 0.0), 2.0, 0.0),
            // Test distance while outside of sphere
            (Vec3::new(2.0, 0.0, 0.0), 1.0, 1.0),
            (Vec3::new(1.5, 0.0, 0.0), 1.0, 0.5),
            // ...with larger radius
            (Vec3::new(3.0, 0.0, 0.0), 2.0, 1.0),
        ];

        // Test raw sampling function
        for case in sample_points.iter() {
            let dist = sample_sphere(case.0, case.1);
            assert_eq!(
                dist, case.2,
                "RAW sample expected {0}, but got {1} | {2} with radius {3}",
                case.2, dist, case.0, case.1
            );
        }

        // Test as sampled from a shape
        for case in sample_points.iter() {
            let sphere = Shape::sphere(Mat4::IDENTITY, case.1);
            let dist = sphere.sample(case.0);
            assert_eq!(
                dist, case.2,
                "SHAPE sample expected {0}, but got {1} | {2} with radius {3}",
                case.2, dist, case.0, case.1
            );
        }
    }

    #[test]
    fn transformed_sample() {
        struct TestCaseTransform {
            note: String,
            sample: Vec3,
            transform: Mat4,
            radius: f32,
            expect: f32,
        }

        let sample_cases: Vec<TestCaseTransform> = vec![
            TestCaseTransform {
                note: String::from("Zero transform, center of sphere"),
                sample: Vec3::ZERO,
                transform: Mat4::IDENTITY,
                radius: 1.0,
                expect: -1.0,
            },

            // Translation
            TestCaseTransform {
                note: String::from("Point is shifted to surface of sphere"),
                sample: Vec3::ZERO,
                transform: Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, Vec3::X),
                radius: 1.0,
                expect: 0.0,
            },
            TestCaseTransform {
                note: String::from("Point is moved forward into center of sphere"),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, Vec3::X),
                radius: 1.0,
                expect: -1.0,
            },
            TestCaseTransform {
                note: String::from("Sphere is moved away from point, so point is above surface of sphere"),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, Vec3::NEG_X),
                radius: 1.0,
                expect: 1.0,
            },

            // Scale
            TestCaseTransform {
                note: String::from("Coordinate space of sphere is half that of the point's, so point position should appear doubled to a regular sphere"),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(Vec3::splat(0.5), Quat::IDENTITY, Vec3::ZERO),
                radius: 1.0,
                expect: 1.0,
            },
            TestCaseTransform {
                note: String::from("Coordinate space of sphere is double that of the point's, so point position should appear half to a regular sphere"),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(Vec3::splat(2.0), Quat::IDENTITY, Vec3::ZERO),
                radius: 1.0,
                expect: -0.5,
            },
        ];

        for case in sample_cases.iter() {
            let sphere = Shape::sphere(case.transform, case.radius);
            let dist = sphere.sample(case.sample);
            let (scale, rot, loc) = case.transform.to_scale_rotation_translation();
            assert_eq!(
                dist, case.expect,
                "SHAPE sample expected {0}, but got {1} | {2} with radius {3} | (scale: {4}, rot: {5}, loc: {6})\n\tcase note: {7}",
                case.expect, dist, case.sample, case.radius, scale, Vec3::from(rot.to_euler(glam::EulerRot::XYZ)), loc, case.note
            );
        }
    }
}
