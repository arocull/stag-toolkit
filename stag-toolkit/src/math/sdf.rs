use crate::math::bounding_box::BoundingBox;
use glam::{Mat4, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles, vec2, vec3};

/// Joins two distance functions, using a logarithm for smoothing values.
/// `k = 32.0`` was the original suggestion for smoothing value.
pub fn smooth_union(a: f32, b: f32, k: f32) -> f32 {
    let res = (-k * a).exp() + (-k * b).exp();
    -res.max(0.0001).log10() / k
}

/// Returns the union of two distance functions: A + B.
pub fn union(a: f32, b: f32) -> f32 {
    a.min(b)
}

/// Returns the intersection of two distance functions: A intersect B.
pub fn intersection(a: f32, b: f32) -> f32 {
    a.max(b)
}

/// Returns the subtraction of two distance functions: A - B.
pub fn subtraction(a: f32, b: f32) -> f32 {
    intersection(a, -b)
}

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

/// Distance function for a rounded cylinder.
///
/// https://iquilezles.org/articles/distfunctions/
/// https://www.shadertoy.com/view/fl3GRl
pub fn sample_cylinder_rounded(
    sample_position: Vec3,
    shape_radius: f32,
    shape_height: f32,
    radius_edge: f32,
) -> f32 {
    let d = vec2(sample_position.xz().length(), sample_position.y.abs())
        - vec2(shape_radius, shape_height * 0.5)
        + Vec2::splat(radius_edge);

    d.max(Vec2::ZERO).length() + d.x.max(d.y).min(0.0) - radius_edge
}

/// Distance function for a torus.
///
/// https://iquilezles.org/articles/distfunctions/
pub fn sample_torus(sample_position: Vec3, ring_thickness: f32, radius: f32) -> f32 {
    let q = vec2(sample_position.xz().length() - radius, sample_position.y);
    q.length() - ring_thickness
}

/// Describes an SDF primitive shape.
#[derive(Copy, Clone, PartialEq)]
pub enum ShapeType {
    /// A sphere primitive.
    Sphere,
    /// A rounded box primitive.
    RoundedBox,
    /// A rounded cylinder primitive.
    RoundedCylinder,
    /// A torus primitive.
    Torus,
}

/// Describes an SDF primitive operation.
#[derive(Copy, Clone, PartialEq)]
pub enum ShapeOperation {
    /// A joining between two shapes.
    Union,
    /// An intersection between two shapes.
    Intersection,
    /// A subtraction between two shapes.
    Subtraction,
}

/// Collection of data describing a Signed Distance Field primitive.
#[derive(Copy, Clone, PartialEq)]
pub struct Shape {
    /// Informs which SDF formula to use when calculating.
    shape: ShapeType,
    /// Informs which operation to use when combining SDFs.
    pub operation: ShapeOperation,
    /// Describes a sphere or cylinder's radius.
    radius: f32,
    /// Describes the edge rounding on the given shape, if available.
    pub radius_ring: f32,
    /// Describes the dimensions of a box or cylinder.
    dimensions: Vec3,
    /// Transform of the shape. Applied to position before sampling.
    transform: Mat4,
    /// Inverse transform of the shape. Used for point projection.
    transform_inv: Mat4,
}

impl Shape {
    /// Creates a sphere primitive with the given parameters.
    pub fn sphere(transform: Mat4, radius: f32, operation: ShapeOperation) -> Self {
        Self {
            shape: ShapeType::Sphere,
            operation,
            transform,
            transform_inv: transform.inverse(),
            radius,
            radius_ring: 0.0,
            dimensions: Vec3::ZERO,
        }
    }
    /// Creates a rounded box primitive with the given parameters.
    pub fn rounded_box(
        transform: Mat4,
        dimensions: Vec3,
        radius_edge: f32,
        operation: ShapeOperation,
    ) -> Self {
        Self {
            shape: ShapeType::RoundedBox,
            operation,
            transform,
            transform_inv: transform.inverse(),
            radius: 0.0,
            radius_ring: radius_edge,
            dimensions,
        }
    }
    /// Creates a rounded cylinder primitive with the given parameters.
    pub fn rounded_cylinder(
        transform: Mat4,
        height: f32,
        radius: f32,
        radius_edge: f32,
        operation: ShapeOperation,
    ) -> Self {
        Self {
            shape: ShapeType::RoundedCylinder,
            operation,
            transform,
            transform_inv: transform.inverse(),
            radius,
            radius_ring: radius_edge,
            dimensions: vec3(1.0, height, 1.0),
        }
    }
    /// Creates a torus primitive with the given parameters.
    /// Uses `radius` for its outer radius, and `radius_edge` for its inner radius.
    pub fn torus(
        transform: Mat4,
        ring_thickness: f32,
        radius: f32,
        operation: ShapeOperation,
    ) -> Self {
        Self {
            shape: ShapeType::Torus,
            operation,
            transform,
            transform_inv: transform.inverse(),
            radius,
            radius_ring: ring_thickness,
            dimensions: Vec3::ONE,
        }
    }
    /// Samples the SDF shape at the given point.
    /// Returned value is the point's distance to the surface of the shape,
    /// with negative being inside the shape, positive being outside.
    pub fn sample(&self, at: Vec3, edge_radius: f32) -> f32 {
        let position_local = self
            .transform_inv
            .mul_vec4(Vec4::new(at.x, at.y, at.z, 1.0))
            .xyz();
        match self.shape {
            ShapeType::Sphere => sample_sphere(position_local, self.radius),
            ShapeType::RoundedBox => {
                sample_box_rounded(position_local, self.dimensions, edge_radius)
            }
            ShapeType::RoundedCylinder => {
                sample_cylinder_rounded(position_local, self.radius, self.dimensions.y, edge_radius)
            }
            ShapeType::Torus => sample_torus(position_local, self.radius_ring, self.radius),
        }
    }
    /// Returns the minimum and maximum boundary points of the shape, NOT transformed
    pub fn relative_bounds(&self) -> BoundingBox {
        match self.shape {
            ShapeType::Sphere => {
                BoundingBox::new(Vec3::splat(-self.radius), Vec3::splat(self.radius))
            }
            ShapeType::RoundedBox => BoundingBox::new(
                self.dimensions * Vec3::splat(-0.5),
                self.dimensions * Vec3::splat(0.5),
            ),
            ShapeType::RoundedCylinder => BoundingBox::new(
                vec3(-self.radius, -self.dimensions.y * 0.5, -self.radius),
                vec3(self.radius, self.dimensions.y * 0.5, self.radius),
            ),
            ShapeType::Torus => {
                let width = self.radius + self.radius_ring;
                BoundingBox::new(
                    vec3(-width, -self.radius_ring, -width),
                    vec3(width, self.radius_ring, width),
                )
            }
        }
    }

    /// Returns the transform of the given shape.
    pub fn transform(&self) -> Mat4 {
        self.transform
    }

    /// Sets the transform of the given shape.
    pub fn set_transform(&mut self, transform: Mat4) {
        self.transform_inv = transform.inverse();
        self.transform = transform;
    }
}

/// Iterates through a shape list, sampling each shape at the given point
/// and smooth unioning the shapes together, returning a distance.
pub fn sample_shape_list(list: &[Shape], point: Vec3, radius_edge: f32) -> f32 {
    let mut d: f32 = 1.0;

    for shape in list.iter() {
        let j = shape.sample(point, radius_edge);

        match shape.operation {
            ShapeOperation::Union => {
                d = union(d, j);
            }
            ShapeOperation::Intersection => {
                d = intersection(d, j);
            }
            ShapeOperation::Subtraction => {
                d = subtraction(d, j);
            }
        }
    }

    d
}

/// Creates an axis-aligned bounding box that encloses all provided Union shapes.
/// If the shape list is empty, returns a zero-volume bounding box centered on (0, 0, 0).
pub fn shape_list_bounds(list: &[Shape]) -> BoundingBox {
    // We use an option here so we don't forcibly enclose 0,0,0
    let mut aabb: Option<BoundingBox> = None;

    for shape in list.iter() {
        if shape.operation == ShapeOperation::Union {
            // Get transformed bounding box of shape
            let shape_aabb = shape.transform() * shape.relative_bounds();

            // If we already set a bounding box, update it to include the shape
            if let Some(unwrapped_aabb) = aabb {
                aabb = Some(unwrapped_aabb.join(&shape_aabb));
            } else {
                // Otherwise, set a new bounding box
                aabb = Some(shape_aabb);
            }
        }
    }

    aabb.unwrap_or_default()
}

// UNIT TESTS //
#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::delta::assert_in_delta;

    use glam::Quat;

    #[test]
    fn test_smooth_union() {
        // Desmos calculator: https://www.desmos.com/calculator/kujnqrwocy
        // A, B, Smoothing (K), expected
        const TEST_MAX_DIFF: f32 = 1e-3f32;
        let cases = [
            (1.0, -1.0, 8.0, -std::f32::consts::LOG10_E),
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
            let sphere = Shape::sphere(Mat4::IDENTITY, case.1, ShapeOperation::Union);
            let dist = sphere.sample(case.0, 0.0);
            assert_eq!(
                dist, case.2,
                "SHAPE sample expected {0}, but got {1} | {2} with radius {3}",
                case.2, dist, case.0, case.1
            );
        }
    }

    #[test]
    fn sdf_cylinder() {
        // Position, radius, height, edge radius, distance
        let sample_points = vec![
            (Vec3::ZERO, 1.0, 1.0, 0.0, -0.5),
            (vec3(1.0, 0.0, 0.0), 1.0, 1.0, 0.0, 0.0), // On side of cylinder
            (vec3(0.0, 0.0, 1.0), 1.0, 1.0, 0.0, 0.0), // On side of cylinder
            (vec3(1.0, 0.0, 1.0).normalize(), 1.0, 1.0, 0.0, 0.0), // On side of cylinder
            (vec3(0.0, 0.5, 0.0), 1.0, 1.0, 0.0, 0.0), // On top of cylinder
            (vec3(0.0, -0.5, 0.0), 1.0, 1.0, 0.0, 0.0), // On bottom of cylinder
            (vec3(2.0, 0.0, 0.0), 1.0, 1.0, 0.0, 1.0), // Far outside cylinder
            (vec3(0.0, 2.0, 0.0), 1.0, 1.0, 0.0, 1.5), // Far above cylinder
            (vec3(0.0, -2.0, 0.0), 1.0, 1.0, 0.0, 1.5), // Far below cylinder
            (vec3(1.0, 0.5, 0.0), 1.0, 1.0, 0.0, 0.0), // Edge of cylinder
            (vec3(1.0, 0.5, 0.0), 1.0, 1.0, 0.5, 0.20710677), // Edge of cylinder with edge radius
            (vec3(1.0, 0.0, 0.0), 1.0, 1.0, 0.25, 0.0), // Side of cylinder with edge radius
        ];

        for case in sample_points.iter() {
            let dist = sample_cylinder_rounded(case.0, case.1, case.2, case.3);

            assert_in_delta(
                case.4,
                dist,
                1e-6,
                format!(
                    "RAW sample expected {0}, but got {1} | {2} with radius {3}, height {4}, border {5}",
                    case.4, dist, case.0, case.1, case.2, case.3,
                ),
            );
        }
    }

    #[test]
    fn sdf_torus() {
        // sample point, inner radius, outer radius, expected distance
        let sample_points = [
            (Vec3::ZERO, 0.5, 1.0, 0.5),           // In center of torus
            (Vec3::ZERO, 0.75, 1.0, 0.25),         // center of torus with larger rings
            (Vec3::ZERO, 1.0, 1.0, 0.0),           // In center of torus with largest rings
            (vec3(1.0, 0.0, 0.0), 0.5, 1.0, -0.5), // in center of ring cross-section
            (vec3(1.5, 0.0, 0.0), 0.5, 1.0, 0.0),  // surface of ring cross-section
            (vec3(2.0, 0.0, 0.0), 0.5, 1.0, 0.5),
        ];

        for case in sample_points.iter() {
            let dist = sample_torus(case.0, case.1, case.2);

            assert_in_delta(
                case.3,
                dist,
                1e-6,
                format!(
                    "RAW sample expected {0}, but got {1} | {2} with inner {3} and outer {4}",
                    case.3, dist, case.0, case.1, case.2,
                ),
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
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::ONE,
                    Quat::IDENTITY,
                    Vec3::X,
                ),
                radius: 1.0,
                expect: 0.0,
            },
            TestCaseTransform {
                note: String::from("Point is moved forward into center of sphere"),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::ONE,
                    Quat::IDENTITY,
                    Vec3::X,
                ),
                radius: 1.0,
                expect: -1.0,
            },
            TestCaseTransform {
                note: String::from(
                    "Sphere is moved away from point, so point is above surface of sphere",
                ),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::ONE,
                    Quat::IDENTITY,
                    Vec3::NEG_X,
                ),
                radius: 1.0,
                expect: 1.0,
            },
            // Scale
            TestCaseTransform {
                note: String::from(
                    "Coordinate space of sphere is half that of the point's, so point position should appear doubled to a regular sphere",
                ),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.5),
                    Quat::IDENTITY,
                    Vec3::ZERO,
                ),
                radius: 1.0,
                expect: 1.0,
            },
            TestCaseTransform {
                note: String::from(
                    "Coordinate space of sphere is double that of the point's, so point position should appear half to a regular sphere",
                ),
                sample: Vec3::X,
                transform: Mat4::from_scale_rotation_translation(
                    Vec3::splat(2.0),
                    Quat::IDENTITY,
                    Vec3::ZERO,
                ),
                radius: 1.0,
                expect: -0.5,
            },
        ];

        for case in sample_cases.iter() {
            let sphere = Shape::sphere(case.transform, case.radius, ShapeOperation::Union);
            let dist = sphere.sample(case.sample, 0.0);
            let (scale, rot, loc) = case.transform.to_scale_rotation_translation();
            assert_eq!(
                dist,
                case.expect,
                "SHAPE sample expected {0}, but got {1} | {2} with radius {3} | (scale: {4}, rot: {5}, loc: {6})\n\tcase note: {7}",
                case.expect,
                dist,
                case.sample,
                case.radius,
                scale,
                Vec3::from(rot.to_euler(glam::EulerRot::XYZ)),
                loc,
                case.note
            );
        }
    }

    #[test]
    fn test_shape_list_bounds() {
        let shapes = vec![
            // Scaled and translated sphere union
            Shape::sphere(
                Mat4::from_scale_rotation_translation(Vec3::splat(0.5), Quat::IDENTITY, Vec3::ONE),
                1.0,
                ShapeOperation::Union,
            ),
            // Non-union shapes are ignored
            Shape::torus(
                Mat4::from_translation(Vec3::NEG_ONE),
                1.0,
                0.5,
                ShapeOperation::Intersection,
            ),
        ];

        // Boundaries should not include 0, 0, 0
        let bounds = shape_list_bounds(&shapes);
        assert_eq!(bounds, BoundingBox::new(Vec3::splat(0.5), Vec3::splat(1.5)));
    }
}
