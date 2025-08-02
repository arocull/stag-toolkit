use glam::{Vec3, Vec4, Vec4Swizzles};

#[derive(Copy, Clone, Default, Debug)]
pub struct RayIntersectionResult {
    /// Intersection point between the ray and the plane.
    pub intersection: Vec3,
    /// If true, this ray collided with the plane in **either** direction.
    pub collided: bool,
    /// If true, the plane normal is parallel to the ray.
    pub reversed: bool,
}

/// Constructs a 3D plane using the given origin and normal values, describing the plane as a 4D vector.
/// To produce a normalized plane, `normal` is expected to be a normalized vector.
pub fn plane(origin: Vec3, normal: Vec3) -> Vec4 {
    Vec4::new(normal.x, normal.y, normal.z, -normal.dot(origin))
}

/// A 3D plane for performing projections.
pub trait Plane {
    /// Returns a new plane with a flipped normal.
    fn flip(self) -> Self;
    /// Returns the signed distance from the given point to this plane.
    fn signed_distance(self, point: Vec3) -> f32;
    /// Intersects the given plane with the given ray, and returns a position and true it sucessfully collided.
    /// If the ray is parallel with the plane, returns the ray origin instead.
    fn ray_intersection(self, ray_origin: Vec3, ray_direction: Vec3) -> RayIntersectionResult;
}

impl Plane for Vec4 {
    fn flip(self) -> Self {
        self * Self::splat(-1.0)
    }

    fn signed_distance(self, point: Vec3) -> f32 {
        self.dot(Self::new(point.x, point.y, point.z, 1.0))
    }

    fn ray_intersection(self, ray_origin: Vec3, ray_direction: Vec3) -> RayIntersectionResult {
        let dt = self.xyz().dot(ray_direction);

        // Test if ray direction is perpendicular to plane normal (parallel)
        if dt == 0.0 {
            return RayIntersectionResult {
                intersection: ray_origin,
                collided: false, // Cast never collides
                reversed: false,
            };
        }

        let projected = ray_origin // Return projected point
                - Vec3::splat(
                    self.dot(Self::new(ray_origin.x, ray_origin.y, ray_origin.z, 1.0))
                        / dt,
                ) * ray_direction;

        RayIntersectionResult {
            intersection: projected,
            collided: true, // Cast successfully collided
            reversed: !dt.is_sign_negative(),
        }
    }
}

/// Finds the index of the point furthest in a given direction from a set of points.
pub fn furthest_point(points: &[Vec3], plane_normal: Vec3, plane_point: Vec3) -> usize {
    let mut max_distance = f32::NEG_INFINITY;
    let mut furthest_index = 0;

    for (i, point) in points.iter().enumerate() {
        let p = plane(plane_point, plane_normal);
        let distance = p.signed_distance(*point).abs();
        if distance > max_distance {
            max_distance = distance;
            furthest_index = i;
        }
    }

    furthest_index
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use super::*;

    #[test]
    fn plane_signed_distance() {
        struct TestPlanePointProject {
            /// Origin point of plane
            origin: Vec3,
            /// Origin normal of plane
            normal: Vec3,
            /// Point to project onto plane
            point: Vec3,
            /// Expected distance to the given point
            distance: f32,
        }

        let cases: Vec<TestPlanePointProject> = vec![
            // Point exists above plane normal
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(0.0, 1.0, 0.0),
                distance: 1.0,
            },
            // Point exists below plane normal
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(0.0, -1.0, 0.0),
                distance: -1.0,
            },
            // Point exists on plane surface
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(0.0, 0.0, 0.0),
                distance: 0.0,
            },
            // Point should be below a flipped plane
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, -1.0, 0.0),
                point: Vec3::new(0.0, 1.0, 0.0),
                distance: -1.0,
            },
            // Point distance remains the same when plane is translated
            TestPlanePointProject {
                origin: Vec3::new(-15.0, 0.0, 15.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(0.0, 1.0, 0.0),
                distance: 1.0,
            },
            // Point distance remains the same when point is translated
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(21.3, 1.0, 31.5),
                distance: 1.0,
            },
            // Point distance at random orientation
            TestPlanePointProject {
                origin: Vec3::new(0.0, 0.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 1.0).normalize(),
                point: Vec3::new(0.0, 1.0, 1.0).normalize() * Vec3::splat(3.0),
                distance: 3.0,
            },
            // Plane at random orientation
            TestPlanePointProject {
                origin: Vec3::new(0.0, 1.0, 0.0),
                normal: Vec3::new(0.0, 1.0, 0.0),
                point: Vec3::new(0.0, 2.0, 0.0),
                distance: 1.0,
            },
        ];

        let max_diff: f32 = 1e-5;
        for (idx, case) in cases.iter().enumerate() {
            let p = plane(case.origin, case.normal);
            let dist = p.signed_distance(case.point);
            let diff = (case.distance - dist).abs();
            assert!(
                (case.distance - dist) < max_diff,
                "Case {0}, expected {1} to be close to {2}, but got difference of {3}",
                idx,
                dist,
                case.distance,
                diff
            );
        }
    }

    #[test]
    fn test_intersect_plane_ray() {
        struct TestPlanePointProject {
            /// Origin point of plane
            o: Vec3,
            /// Origin normal of plane
            n: Vec3,
            /// Ray origin to project onto plane
            ro: Vec3,
            /// Ray direction to project onto plane
            rd: Vec3,
            result: Vec3,
            collided: bool,
            reverse: bool,
        }

        let test_cases: Vec<TestPlanePointProject> = vec![
            // Above plane, casting to origin
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::NEG_Z,
                result: Vec3::ZERO,
                collided: true,
                reverse: false,
            },
            // Above plane, casting away from origin. Should hit plane anyway
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::Z,
                result: Vec3::ZERO,
                collided: true,
                reverse: true,
            },
            // Above plane, casting parallel to it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::X,
                result: Vec3::Z,
                collided: false,
                reverse: false,
            },
            // Below plane, casting to it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::NEG_Z,
                rd: Vec3::Z,
                result: Vec3::ZERO,
                collided: true,
                reverse: true,
            },
            // Below plane, casting away from it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::NEG_Z,
                rd: Vec3::NEG_Z,
                result: Vec3::ZERO,
                collided: true,
                reverse: false,
            },
            // Plane has origin offset
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z * 2.0,
                rd: Vec3::Z,
                result: Vec3::Z,
                collided: true,
                reverse: true,
            },
            // Ray origin is on plane
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::Z,
                result: Vec3::Z,
                collided: true,
                reverse: true,
            },
            // Ray origin is on plane, and parallel to it
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::X,
                result: Vec3::Z,
                collided: false,
                reverse: false,
            },
        ];

        for (idx, case) in test_cases.iter().enumerate() {
            let pl = plane(case.o, case.n);
            let result = pl.ray_intersection(case.ro, case.rd);

            assert_eq!(
                result.collided, case.collided,
                "case {idx}: collision state should max expected"
            );

            assert_eq!(
                result.intersection, case.result,
                "case {0}: ray [{1} -> {2}) did not project onto\t{3}",
                idx, case.ro, case.rd, pl
            );

            assert_eq!(
                result.reversed, case.reverse,
                "case {0}: ray [{1} -> {2}) did not match backface",
                idx, case.ro, case.rd,
            )
        }
    }
}
