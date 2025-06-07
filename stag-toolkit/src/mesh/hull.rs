use std::{collections::HashMap, f32::EPSILON, ptr::null_mut};

use crate::{math::projection::{plane, Plane}, mesh::trimesh::Edge};
use glam::{Vec3, Vec4};
use godot::global::godot_warn;

use super::{
    pointcloud::PointCloud,
    trimesh::{Triangle, TriangleOperations},
};

struct QuickHullItem {
    triangle: Triangle,
    covered: Vec<usize>,
    plane: Vec4,
}
impl QuickHullItem {
    fn new(triangle: Triangle, pla: Vec4) -> Self {
        Self {
            triangle,
            covered: vec![],
            plane: pla,
        }
    }
}

struct TriConnector<'a> {
    left: &'a mut QuickHullItem,
    right: &'a mut QuickHullItem,
}

/// Generates a convex hull encapsulating the Point Cloud, using the Quick Hull algorithm.
///
/// - Returned triangle array directly references the provided Point Cloud.
/// - Neither Point Cloud nor resulting mesh are optimized during or after generation.
///
/// Mirrors Godot's implementation of QuickHull.
pub fn quick_hull(points: &Vec<Vec3>) -> Option<Vec<Triangle>> {
    let aabb = points.bounds();
    if aabb.size.length() <= 0.001 {
        godot_warn!("StagToolkit: Bounds too small to create a quick hull.");
        return None;
    }

    let simplex: [usize; 4];

    // Find initial points for convex hull
    {
        // Get two points that are most distant from each other
        let (idx_smallest, idx_largest) = points.distant(aabb);

        // Get furthest point from the constructed line
        let furthest_from_line = points.distant_line(points[idx_smallest], points[idx_largest]);

        // Get furthest point from the constructed plane
        let tri: Triangle = [idx_smallest, idx_largest, furthest_from_line];
        let p = plane(points[idx_smallest], tri.normal(points));

        let furthest_from_plane = points.distant_plane(p);

        simplex = [
            idx_smallest,
            idx_largest,
            furthest_from_line,
            furthest_from_plane,
        ];
    }


    // Get centerpoint of simplex
    let center =
        (points[simplex[0]] + points[simplex[1]] + points[simplex[2]] + points[simplex[3]])
            * Vec3::splat(0.25);

    // Generate faces for simplex
    let mut faces: Vec<QuickHullItem> = vec![];
    faces.reserve(4);

    // Create initial convex hull
    const FACE_ORDER: [[usize; 3]; 4] = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]];
    for i in 0..4 {
        // Create a triangle for the given point order
        let mut tri: Triangle = [FACE_ORDER[i][0], FACE_ORDER[i][1], FACE_ORDER[i][2]];

        // If the triangle does not face away from the centerpoint, flip it
        if !tri.is_point_behind(points, center) {
            tri = tri.flip();
        }

        faces.push(QuickHullItem::new(tri, tri.plane(points)));
    }

    let tolerance = 3.0 * EPSILON * (aabb.size.x + aabb.size.y + aabb.size.z);

    // Find all points behind the given face
    for (idx, pt) in points.iter().enumerate() {
        for face in faces.iter_mut() {
            // If the given point is behind the plane within a set tolerance
            // indicate that the point is contained
            if face.plane.signed_distance(*pt) < tolerance {
                face.covered.push(idx);
                break;
            }
        }
    }

    // AAAA
    let mut max_iterations = 1000000;
    while max_iterations > 0 && faces[faces.len() - 1].covered.len() > 0 {
        max_iterations -= 1;

        let last_face = &faces[faces.len() - 1];

        // Find vertex most outside of face
        let mut next = 0;
        let mut next_dist: f32 = 0.0;

        for (idx, pt) in last_face.covered.iter().enumerate() {
            let dist = last_face.plane.signed_distance(points[*pt]);

            if dist > next_dist {
                next_dist = dist;
                next = idx;
            }
        }

        // Most distant vertex
        let v = points[last_face.covered[next]];

        // Find lit and lit edges
        let lit_faces: Vec<Triangle> = vec![];
        let lit_edges: HashMap<Edge, TriConnector> = HashMap::new();

        for tri in faces.iter_mut() {
            if tri.triangle.plane(points).signed_distance(v) > 0 {
                lit_faces.push(tri.triangle);

                for i in 0..3 {
                    let a = tri.triangle[i];
                    let b = tri.triangle[(i + 1) % tri.triangle.len()];
                    let edge: Edge = [a, b];

                    let connector_opt = lit_edges.get(&edge);
                    let connector: TriConnector;
                    match connector_opt {
                        Some(conn) => {

                        },
                        None => {
                            connector = TriConnector {
                                left: null_mut(),
                                right: null_mut(),
                            }
                        }
                    }
                    if !lit_edges.contains_key(&edge) {
                        let connector = *lit_edges.get(&edge);

                        if edge[0] == a {
                            connector.left = tri;
                        } else {
                            connector.right = tri;
                        }

                        continue;
                    }


                    let connector = TriConnector {
                        left: tri,
                        right: tri,
                    };
                }
            }
        }

    }

    // TODO: do more quickhull stuff

    // finally, return list of all faces
    let tris = faces
        .iter()
        .map(|val| -> Triangle { val.triangle })
        .collect();

    Some(tris)
}

#[cfg(test)]
mod tests {
    use crate::{math::types::Vec3, mesh::hull::quick_hull};

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
        let hull = quick_hull(&pts).unwrap();

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
