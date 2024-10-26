use crate::math::projection::furthest_point;
use glam::Vec3;

use super::trimesh::{Triangle, TriangleOperations};

/// Returns the index values of points with the minimum X, maximum X, Y, and minimum Y respectively.
fn find_extremes(points: &Vec<Vec3>) -> (usize, usize, usize) {
    let mut min_x: usize = 0;
    let mut max_x: usize = 0;
    let mut min_y: usize = 0;

    for (idx, pt) in points.iter().enumerate() {
        if pt.x < points[min_x].x {
            min_x = idx;
        } else if pt.x > points[max_x].x {
            max_x = idx;
        }
        if pt.y < points[min_y].y {
            min_y = idx;
        }
    }

    (min_x, max_x, min_y)
}

/// Quick Hull initialization step, find the 3 furthermost bounding points to start our hull from.
/// TODO
fn convex_hull_initialize(points: &Vec<Vec3>) -> Vec<Triangle> {
    let (min_x, max_x, min_y) = find_extremes(points);

    let base_triangle: Triangle = [min_x, min_y, max_x];
    let norm = base_triangle.normal(points);
    let furthest = furthest_point(points, norm, points[min_x]);

    let tris: Vec<Triangle> = vec![
        base_triangle,
        [min_x, furthest, min_y],
        [min_y, furthest, max_x],
        [min_x, furthest, max_x],
    ];

    tris
}

/// Quick Hull implementation for convex hull agorithms.
pub fn convex_hull(points: &Vec<Vec3>) -> Vec<Triangle> {
    // Construct the initial convex hull
    let mut hull = convex_hull_initialize(points);

    for (idx, p) in points.iter().enumerate() {
        // Find visible faces from point 'p'
        let visible_faces: Vec<Triangle> = hull
            .iter()
            .filter(|face| face.is_point_behind(points, *p)) // Replace with intersection check logic
            .cloned()
            .collect();

        // Create new triangles and remove old ones
        for face in visible_faces {
            let a = face[0];
            let b = face[1];
            let c = face[2];

            hull.push([a, b, idx]);
            hull.push([b, c, idx]);

            // Remove the old triangle from the hull
            hull.retain(|f| !(f.eq(&face)));
        }
    }

    hull
}

#[cfg(test)]
mod tests {

    use super::{convex_hull, convex_hull_initialize};
    use crate::math::types::Vec3;

    #[ignore]
    #[test]
    fn test_convex_hull_initialize() {
        // Define initial points for hulling
        let pts: Vec<Vec3> = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::ZERO, // Point should not be contained within the hull
        ];

        let initial_hull = convex_hull_initialize(&pts);
        assert_eq!(initial_hull.len(), 4, "initial hull should be 4 triangles");

        assert_ne!(
            initial_hull[0][0], initial_hull[0][1],
            "of first triangle, first and second point should not be the same"
        );
        assert_ne!(
            initial_hull[0][0], initial_hull[0][2],
            "of first triangle, first and last point should not be the same"
        );
        assert_ne!(
            initial_hull[0][1], initial_hull[0][2],
            "of first triangle, second and last point should not be the same"
        );
    }

    #[ignore]
    #[test]
    fn test_convex_hull() {
        // Define initial points for hulling
        let pts: Vec<Vec3> = vec![
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::ZERO, // Point should not be contained within the hull
        ];
        // ...and whether the given point should be contained
        let should_contain: Vec<bool> = vec![true, true, true, true, true, true, true, true, false];
        let mut does_contain: Vec<bool> = vec![false; should_contain.len()];

        // Perform convex hull algorithm
        let hull = convex_hull(&pts);

        assert!(
            hull.len() >= 4,
            "Hull should be 4 triangles at minimum, but got {0} triangle(s).\nhull: {1:?}",
            hull.len(),
            hull
        );

        // Validate what points are in the hull
        for tri in hull.iter() {
            for idx in tri {
                does_contain[*idx] = true;
            }
        }

        assert_eq!(
            does_contain, should_contain,
            "hull should only contain expected points\nhull: {0:?}",
            hull
        );
    }
}
