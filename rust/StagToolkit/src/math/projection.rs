use glam::{Vec3, Vec4, Vec4Swizzles};

/// Constructs a 3D plane using the given origin and normal values,
/// describing the plane as a 4D vector.
pub fn plane(origin: Vec3, normal: Vec3) -> Vec4 {
    Vec4::new(normal.x, normal.y, normal.z, -normal.dot(origin))
}
/// Computes the distance from a point to a plane
pub fn distance_to_plane(point: Vec3, plane_normal: Vec3, plane_point: Vec3) -> f32 {
    (point - plane_point).dot(plane_normal)
}
/// Intersects the given plane with the given ray, and returns a position and whether it sucessfully collided.
/// If the ray is parallel with the plane, returns the ray origin instead.
pub fn intersect_plane_ray(plane: Vec4, ray_origin: Vec3, ray_direction: Vec3) -> (Vec3, bool) {
    // Test if ray direction is perpendicular to plane normal (parallel)
    if plane.xyz().dot(ray_direction) == 0.0 {
        return (ray_origin, false); // Cast never collides
    }

    (
        ray_origin // Return projected point
            - Vec3::splat(
                plane.dot(Vec4::new(ray_origin.x, ray_origin.y, ray_origin.z, 1.0))
                    / plane.xyz().dot(ray_direction),
            ) * ray_direction,
        true, // Cast successfully collided
    )
}
/// Finds the index of the point furthest in a given direction from a set of points.
pub fn furthest_point(points: &Vec<Vec3>, plane_normal: Vec3, plane_point: Vec3) -> usize {
    let mut max_distance = f32::NEG_INFINITY;
    let mut furthest_index = 0;

    for (i, point) in points.iter().enumerate() {
        let distance = distance_to_plane(*point, plane_normal, plane_point).abs();
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

    use crate::math::projection::{intersect_plane_ray, plane};

    use super::distance_to_plane;

    #[test]
    fn test_distance_to_plane() {
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
        ];

        let max_diff: f32 = 1e-5;
        let mut idx = 0;
        for case in cases.iter() {
            let dist = distance_to_plane(case.point, case.normal, case.origin);
            let diff = (case.distance - dist).abs();
            assert!(
                (case.distance - dist) < max_diff,
                "Case {0}, expected {1} to be close to {2}, but got difference of {3}",
                idx,
                dist,
                case.distance,
                diff
            );
            idx += 1;
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
            },
            // Above plane, casting away from origin. Should hit plane anyway
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::Z,
                result: Vec3::ZERO,
                collided: true,
            },
            // Above plane, casting parallel to it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::X,
                result: Vec3::Z,
                collided: false,
            },
            // Below plane, casting to it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::NEG_Z,
                rd: Vec3::Z,
                result: Vec3::ZERO,
                collided: true,
            },
            // Below plane, casting away from it
            TestPlanePointProject {
                o: Vec3::ZERO,
                n: Vec3::Z,
                ro: Vec3::NEG_Z,
                rd: Vec3::NEG_Z,
                result: Vec3::ZERO,
                collided: true,
            },
            // Plane has origin offset
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z * 2.0,
                rd: Vec3::Z,
                result: Vec3::Z,
                collided: true,
            },
            // Ray origin is on plane
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::Z,
                result: Vec3::Z,
                collided: true,
            },
            // Ray origin is on plane, and parallel to it
            TestPlanePointProject {
                o: Vec3::Z,
                n: Vec3::Z,
                ro: Vec3::Z,
                rd: Vec3::X,
                result: Vec3::Z,
                collided: false,
            },
        ];

        let mut idx = 0;
        for case in test_cases.iter() {
            let pl = plane(case.o, case.n);
            let (projected, hit) = intersect_plane_ray(pl, case.ro, case.rd);

            assert_eq!(
                hit, case.collided,
                "case {0}: collision state should max expected",
                idx
            );

            assert_eq!(
                projected, case.result,
                "case {0}: ray [{1} -> {2}) did not project onto\t{3}",
                idx, case.ro, case.rd, pl
            );

            idx += 1;
        }
    }
}
