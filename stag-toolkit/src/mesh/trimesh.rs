use crate::math::raycast::{Raycast, RaycastParameters, RaycastResult};
use crate::math::{
    projection::{Plane, plane},
    types::*,
};
use glam::Vec4Swizzles;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;
use std::num::NonZero;
// EDGES //

/// A mesh edge of vertex indices. In counter-clockwise winding order.
pub type Edge = [usize; 2];

/// A set of two indices that can be operated from any set of positions.
pub trait EdgeOperations {
    /// Returns a new, flipped edge by changing vertex order.
    fn flip(&self) -> Self;
    /// Returns the calculated length of an edge.
    fn length(&self, positions: &[Vec3]) -> f32;
}

impl EdgeOperations for Edge {
    fn flip(&self) -> Self {
        [self[1], self[0]]
    }

    fn length(&self, positions: &[Vec3]) -> f32 {
        positions[self[0]].distance(positions[self[1]])
    }
}

// TRIANGLES //

/// A mesh triangle of vertex indices. In counter-clockwise face winding order.
pub type Triangle = [usize; 3];

/// A set of three indices that can be operated on from any set of positions.
pub trait TriangleOperations {
    /// Returns a positive value if the triangle's points are oriented counter-clockwise,
    /// negative if clockwise, and zero if they are collinear.
    fn orientation(&self, positions: &[Vec3]) -> f32;
    /// Returns the calculated normal of the given face using a counter-clockwise wound triangle.
    fn normal(&self, positions: &[Vec3]) -> Vec3;
    /// Returns the face plane.
    fn plane(&self, positions: &[Vec3]) -> Vec4;
    /// Projects the given point onto the triangle.
    fn project(&self, positions: &[Vec3], point: Vec3) -> Vec3;
    /// Calculates the projected barycentric coordinates of a point `p` relative to this triangle.
    fn barycentric(&self, positions: &[Vec3], project: Vec3) -> Vec3;
    /// Returns true if the given Barycentric point is contained by the triangle.
    fn contains_barycentric(&self, barycentric_point: Vec3) -> bool;
    /// Returns true if the given point is behind the surface of the triangle.
    fn is_point_behind(&self, positions: &[Vec3], project: Vec3) -> bool;
    /// Returns true if two triangles are the same.
    fn equals(&self, other: &Triangle) -> bool;
    /// Returns a new, flipped triangle by changing vertex order.
    fn flip(&self) -> Self;
    /// Returns true if the triangle has this edge in its specified direction. False otherwise.
    fn has_edge(&self, edge: &Edge) -> bool;
    /// Returns the centerpoint of the triangle.
    fn centerpoint(&self, positions: &[Vec3]) -> Vec3;
    /// Returns the area of the triangle.
    fn area(&self, positions: &[Vec3]) -> f32;
    /// Returns a face-winded list of edges on this triangle.
    fn edges(&self) -> [Edge; 3];
}

impl TriangleOperations for Triangle {
    fn orientation(&self, positions: &[Vec3]) -> f32 {
        (positions[self[1]] - positions[self[0]])
            .cross(positions[self[2]] - positions[self[0]])
            .dot(Vec3::ONE)
    }

    fn normal(&self, positions: &[Vec3]) -> Vec3 {
        let u = positions[self[1]] - positions[self[0]];
        let v = positions[self[2]] - positions[self[0]];
        let c = u.cross(v);

        let len = c.length_squared();

        if len <= 1e-6 {
            // Make sure vector isn't zero-length
            return Vec3::Y; // Default to up if so
        }

        c / len.sqrt() // Return normalized vector
    }

    fn plane(&self, positions: &[Vec3]) -> Vec4 {
        plane(positions[self[0]], self.normal(positions))
    }

    fn project(&self, positions: &[Vec3], point: Vec3) -> Vec3 {
        // Get plane normal
        let norm = self.normal(positions);
        // Create a plane
        let pl = plane(positions[self[0]], norm);
        // Project point onto plane, using opposite of plane's normal.
        // Projection should never fail as ray is always antiparallel to the normal.
        pl.ray_intersection(point, -norm).intersection
    }

    fn barycentric(&self, positions: &[Vec3], project: Vec3) -> Vec3 {
        let a = positions[self[0]];
        let b = positions[self[1]];
        let c = positions[self[2]];

        // Compute vectors and determinants for barycentric calculation
        let v0 = c - a;
        let v1 = b - a;
        let v2 = project - a;

        let d00 = v0.dot(v0);
        let d01 = v0.dot(v1);
        let d11 = v1.dot(v1);
        let d20 = v2.dot(v0);
        let d21 = v2.dot(v1);

        // Calculate barycentric coordinates
        let denom = d00 * d11 - d01 * d01;
        let v = (d11 * d20 - d01 * d21) / denom;
        let w = (d00 * d21 - d01 * d20) / denom;

        Vec3::new(1.0 - v - w, v, w) // Return UVW coordinates
    }

    fn contains_barycentric(&self, barycentric_point: Vec3) -> bool {
        barycentric_point.x >= 0.0
            && barycentric_point.x <= 1.0
            && barycentric_point.y >= 0.0
            && barycentric_point.y <= 1.0
            && barycentric_point.z >= 0.0
            && barycentric_point.z <= 1.0
    }

    fn is_point_behind(&self, positions: &[Vec3], project: Vec3) -> bool {
        let p = self.plane(positions);
        // If our point is above the plane or parallel to it, it is not in front of us
        let d = p.signed_distance(project);
        if d >= 0.0 {
            return false;
        }

        // If our point is behind us, confirm it's behind the triangle itself
        self.contains_barycentric(self.barycentric(positions, self.project(positions, project)))
    }

    fn equals(&self, other: &Triangle) -> bool {
        self[0] == other[0] && self[1] == other[1] && self[2] == other[2]
    }

    fn flip(&self) -> Self {
        [self[1], self[0], self[2]]
    }

    fn has_edge(&self, edge: &Edge) -> bool {
        for i in 0..3 {
            // Check if the edge exists along this point or the next wrapped one
            if edge[0] == self[i] && edge[1] == self[(i + 1) % self.len()] {
                return true;
            }
        }

        false
    }

    fn centerpoint(&self, positions: &[Vec3]) -> Vec3 {
        (positions[self[0]] + positions[self[1]] + positions[self[2]]) * Vec3::splat(1.0 / 3.0)
    }

    fn area(&self, positions: &[Vec3]) -> f32 {
        // https://math.stackexchange.com/questions/128991/how-to-calculate-the-area-of-a-3d-triangle
        let ab = positions[self[0]] - positions[self[1]];
        let ac = positions[self[0]] - positions[self[2]];
        ab.cross(ac).length() * 0.5
    }

    fn edges(&self) -> [Edge; 3] {
        [[self[0], self[1]], [self[1], self[2]], [self[2], self[0]]]
    }
}

// MESHES //

/// An edge with a face (index 0), that may or may not have a corresponding face on the reversed edge (index 1).
pub type EdgeTriangles = (usize, Option<NonZero<usize>>);

/// Container for triangle mesh data.
#[derive(Clone, PartialEq, Default)]
pub struct TriangleMesh {
    /// Primary mesh buffer, listing the index of corresponding vertex positions and normals, in counter-clockwise face winding.
    pub triangles: Vec<Triangle>,
    // pub indices: Vec<usize>,
    /// Individual vertices of the mesh.
    pub positions: Vec<Vec3>,
    /// Normals of the mesh, assigned to vertices of the corresponding index.
    pub normals: Vec<Vec3>,
    /// Optional color data, assigned to vertices of the corresponding index.
    pub colors: Vec<Vec4>,

    pub uv1: Option<Vec<Vec2>>,
    pub uv2: Option<Vec<Vec2>>,
}

impl TriangleMesh {
    /// Creates a new TriangleMesh from the given mesh data.
    pub fn new(
        triangles: Vec<Triangle>,
        positions: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
        colors: Option<Vec<Vec4>>,
    ) -> Self {
        Self {
            triangles,
            positions,
            // Default normals to an empty vector if not included
            normals: normals.unwrap_or_default(),
            colors: colors.unwrap_or_default(),
            uv1: None,
            uv2: None,
        }
    }

    /// Creates a new TriangleMesh from a list of indices.
    /// Every three indices are expected to represent a triangle, with counter-clockwise face winding.
    /// Each index has a corresponding vertex position and normal.
    /// A list of normals is optional.
    pub fn from_indices(
        indices: Vec<usize>,
        positions: Vec<Vec3>,
        normals: Option<Vec<Vec3>>,
    ) -> Self {
        // Reserve triangles
        let mut tris: Vec<Triangle> = Vec::with_capacity(indices.len() / 3);

        // Create triangles for each index
        for i in 0..(indices.len() / 3) {
            tris.push([indices[i * 3], indices[i * 3 + 1], indices[i * 3 + 2]]);
        }

        Self {
            triangles: tris,
            positions,
            // Default normals to an empty vector if not included
            normals: normals.unwrap_or_default(),
            colors: vec![],
            uv1: None,
            uv2: None,
        }
    }

    /// Joins the given mesh with this one, in place.
    /// Does not merge points or optimize the mesh in any way.
    pub fn join(&mut self, mesh: &Self) {
        let idx_count = self.positions.len();

        // Glomp in the other mesh's positions and normals
        self.positions.append(&mut mesh.positions.clone());
        self.normals.append(&mut mesh.normals.clone());

        // Prepare to add a ton of triangles
        self.triangles.reserve_exact(mesh.triangles.len());

        // Shift each triangle index based on our current number of points
        for tri in mesh.triangles.iter() {
            self.triangles
                .push([tri[0] + idx_count, tri[1] + idx_count, tri[2] + idx_count]);
        }
    }

    /// Returns the first left and right faces of an edge, if they exist.
    /// Note: very slow, prefer using `edge_map` instead if handling many edges.
    pub fn tris_for_edge(&self, edge: &Edge) -> (Option<Triangle>, Option<Triangle>) {
        let mut left: Option<Triangle> = None;
        let mut right: Option<Triangle> = None;
        let flip = edge.flip();

        for tri in self.triangles.iter() {
            if tri.has_edge(edge) {
                right = Some(*tri);
                break;
            }
        }
        for tri in self.triangles.iter() {
            if tri.has_edge(&flip) {
                left = Some(*tri);
                break;
            }
        }

        (left, right)
    }

    /// Returns the number of vertex positions in the mesh.
    pub fn count_vertices(&self) -> usize {
        self.positions.len()
    }

    /// Unwraps the triangles in the mesh and returns them as an ordered list of indices.
    pub fn indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = vec![];
        indices.reserve_exact(self.triangles.len());

        for tri in self.triangles.iter() {
            for idx in tri {
                indices.push(*idx);
            }
        }

        indices
    }

    /// Returns a hash map of edges.
    /// For each edge, the left and right face index is returned, in order.
    /// This method assumes that each edge has a maximum of two faces,
    /// but it does not expect the mesh to be watertight.
    pub fn edge_map(&self) -> HashMap<Edge, EdgeTriangles> {
        let mut edges = HashMap::<Edge, EdgeTriangles>::new();

        for (idx, tri) in self.triangles.iter().enumerate() {
            for edge in tri.edges() {
                // Check for reverse-edge first
                let flip = edge.flip();

                // If we found a key already existing for our reverse edge,
                // fill in our face index as the right face
                if let Some(faces) = edges.get_mut(&flip) {
                    #[cfg(debug_assertions)]
                    assert_ne!(0, idx, "face index on reverse edge should never be zero");

                    faces.1 = NonZero::new(idx);
                } else {
                    // Otherwise, insert a key with this face for our edge
                    edges.insert(edge, (idx, None));
                }
            }
        }

        edges
    }

    /// Calculates the angle between two faces.
    pub fn face_angle(&self, a: &Triangle, b: &Triangle) -> f32 {
        a.normal(&self.positions)
            .angle_between(b.normal(&self.positions))
    }

    /// Removes an edge from the mesh by merging both vertices into a centerpoint.
    /// Does not remove degenerate geometry.
    pub fn edge_collapse(&mut self, edge: &Edge) {
        // Create a new vertex at the center of the edge
        let center = (self.positions[edge[0]] + self.positions[edge[1]]) * 0.5;

        // Append vertex to end of positions list
        let new_idx = self.positions.len();
        self.positions.push(center);

        // Swap out old vertex indices for new one
        self.swap_indices(vec![(edge[0], new_idx), (edge[1], new_idx)]);
    }

    /// Decimates the mesh by removing all immediate edges with an angle less than the given threshold.
    /// When the number of triangles removed per decimation falls under the `minimum_dropout` threshold,
    /// the algorithm stops decimating triangles.
    pub fn decimate_planar(&mut self, threshold: f32, iterations: u32, minimum_dropout: u32) {
        // Do nothing if invalid.
        if iterations == 0 {
            return;
        }

        for _ in 0..iterations {
            // Get a list of all edges in the trimesh
            let edges = self.edge_map();

            // Collapse all edges below the threshold
            let mut count = 0;
            for (edge, (left_idx, right_idx)) in edges.iter() {
                if let Some(right_idx) = right_idx
                    && self.face_angle(&self.triangles[*left_idx], &self.triangles[right_idx.get()])
                        < threshold
                {
                    self.edge_collapse(edge);
                    count += 1;
                }
            }

            // Clean up mesh after decimation
            self.remove_degenerate();

            // End decimation if nothing changed
            if count <= minimum_dropout {
                break;
            }
        }

        self.remove_unused();
    }

    /// Merges all vertices within the given threshold distance of each other, merging later vertices into earlier ones.
    /// This operation occurs in place.
    ///
    /// **Does not remove degenerate triangles or unused vertices.**
    /// Call `remove_degenerate` and `remove_unused` to clean up the mesh when you are done editing it.
    /// Or, to do everything at once, call `optimize`.
    pub fn merge_by_distance(&mut self, threshold: f32) {
        if threshold <= 0.0 {
            // Don't do anything if disabled
            return;
        }

        let thresh_squared = threshold * threshold;

        // Array of new, merged vertices
        let mut new_verts = self.positions.clone();
        // List of vertex indices: (replace, new)
        // Estimate that we'll roughly need 10% of our vertex list to deal with
        let mut replace: Vec<(usize, usize)> =
            Vec::with_capacity((new_verts.len() as f64 * 0.1) as usize);

        // Start from the back of the array
        for (i, vert) in self.positions.iter().enumerate().rev() {
            // ...read forward until we hit our current index
            for j in 0..i {
                if vert.distance_squared(new_verts[j]) <= thresh_squared {
                    // Remove vertices at the back of the new list
                    new_verts.remove(i);
                    // ...and modify the vertices at the front to be the midpoint
                    new_verts[j] = (vert + new_verts[j]) * 0.5;

                    // ...and note what vertices to replace
                    replace.push((i, j));

                    break;
                }
            }
        }

        // Finally, update triangle indices
        self.swap_indices(replace);
    }

    /// Iterates over all triangles, replacing each vertex index value using the given tuple: (old, new).
    /// Does not remove degenerate triangles.
    pub fn swap_indices(&mut self, replace: Vec<(usize, usize)>) {
        if replace.is_empty() {
            return;
        }

        // Iterate over every swap item
        for idx_swap in replace.iter() {
            for tri in self.triangles.iter_mut() {
                // Update the triangle indices
                for idx in tri.iter_mut() {
                    if idx_swap.0 == *idx {
                        *idx = idx_swap.1;
                    }
                }
            }
        }
    }

    /// Removes degenerate triangles from the mesh.
    pub fn remove_degenerate(&mut self) {
        // Ensure no vertex indices on the triangle match
        self.triangles
            .retain(|tri| !(tri[0] == tri[1] || tri[0] == tri[2] || tri[1] == tri[2]));
    }

    /// Removes all unused vertex positions in the mesh.
    pub fn remove_unused(&mut self) {
        // Keep track of all used points
        let mut used: Vec<bool> = vec![];
        used.resize(self.positions.len(), false);

        // Figure out what points are used
        for tri in self.triangles.iter() {
            for item in tri {
                used[*item] = true;
            }
        }

        // Drop points that are not associated with anything
        let mut idx: usize = 0;
        self.positions.retain(|_item| {
            let i = idx;
            idx += 1;
            used[i]
        });

        // ...do the same for normals.
        idx = 0;
        self.normals.retain(|_item| {
            let i = idx;
            idx += 1;
            used[i]
        });

        // Create an array for remapping vertex index values
        let mut remapped: Vec<usize> = vec![0; used.len()];
        let mut new_idx: usize = 0; // Current available index
        for (idx, used) in used.iter().enumerate() {
            if *used {
                remapped[idx] = new_idx;
                new_idx += 1;
            }
        }

        // Adjust all triangle indices accordingly
        let mut new_tris: Vec<Triangle> = vec![];
        new_tris.reserve_exact(self.triangles.len());

        for tri in self.triangles.iter() {
            new_tris.push([remapped[tri[0]], remapped[tri[1]], remapped[tri[2]]]);
        }

        self.triangles = new_tris;
    }

    /// Calculates smooth vertex normals, using each triangle's surface area as a weight.
    /// Returns as a list of surface normals for each corresponding vertex.
    pub fn get_normals_smooth(&self) -> Vec<Vec3> {
        // Map of vertices, with a list of corresponding triangle normals and associated area
        let mut vertices: HashMap<usize, Vec<(Vec3, f32)>> = HashMap::new();

        for tri in self.triangles.iter() {
            let norm = tri.normal(&self.positions);
            let area = tri.area(&self.positions);

            for idx in tri.iter() {
                if let Some(vertex_normals) = vertices.get_mut(idx) {
                    vertex_normals.push((norm, area));
                } else {
                    vertices.insert(*idx, vec![(norm, area)]);
                }
            }
        }

        // Allocate new normal buffer and fill with default values
        let mut normals: Vec<Vec3> = Vec::with_capacity(vertices.len());
        normals.resize(vertices.len(), Vec3::ZERO);

        // Fill in normals based on triangle normals weighted by triangle area
        for (idx, vertex_data) in vertices {
            let mut total_area: f32 = 0.0;
            for (_, area) in vertex_data.iter() {
                total_area += area;
            }

            let mut vertex_normal = Vec3::ZERO;
            for (normal, area) in vertex_data.iter() {
                vertex_normal += normal * (area / total_area);
            }
            normals[idx] = -vertex_normal.normalize_or_zero();
        }

        normals
    }

    // Computes a corresponding normal for each mesh vertex by sampling a list of SDF shapes.
    // pub fn get_normals_sdf(&self) -> Vec<Vec3> {
    //     vec![]
    // }

    /// Bakes out smooth vertex normals, using each triangle's surface area as a weight.
    pub fn bake_normals_smooth(&mut self) {
        self.normals = self.get_normals_smooth();
    }

    /// Computes and returns an ambient occlusion for every vertex on the mesh.
    /// Requires vertex normals to be baked beforehand.
    /// This occlusion method is based on raycasting.
    pub fn get_ambient_occlusion(&self, samples: usize, radius: f32, seed: u32) -> Vec<f32> {
        let mut occlusion: Vec<f32> = Vec::with_capacity(self.positions.len());

        let perlin = Perlin::new(seed);

        #[cfg(debug_assertions)]
        assert!(
            self.normals.len() >= self.positions.len(),
            "each vertex must have a corresponding normal"
        );

        // TODO: multithread this via rayon

        let radius_squared = radius * radius;

        for (idx, pt) in self.positions.iter().enumerate() {
            let normal = self.normals.get(idx).unwrap_or(&Vec3::ZERO);
            // TODO: random direction in cone

            let orientation = Quat::look_to_rh(*normal, Vec3::Y);

            let mut results: Vec<f32> = Vec::with_capacity(samples);

            for iteration in 0..samples {
                // let z = perlin.get([pt.x as f64, pt.y as f64, pt.z as f64, iteration as f64]).remap(-1.0,1.0,0.0,1.0);
                // let theta = perlin.get([pt.x as f64, pt.y as f64, pt.z as f64, (iteration * samples) as f64]);
                // let dir = vector_in_cone(orientation, z as f32, theta.remap(-1.0, 1.0, 0.0, TAU) as f32);

                let origin = pt - normal * 1000.0;
                let params = RaycastParameters::new(origin, *normal, f32::INFINITY, false);

                // If we hit, store inverse of linear falloff from center to edge
                if let Some(result) = self.raycast(params) {
                    let distance_squared = result.point.distance_squared(*pt);
                    if distance_squared < radius_squared {
                        results.push(1.0 - (distance_squared.sqrt() / radius));
                    }
                }
            }

            // Average results and then sqrt the proportion so it leans toward lighter
            let count = results.len();
            if count > 0 {
                let proportion = results.iter().sum::<f32>() / count as f32;
                occlusion.push(proportion.sqrt());
            } else {
                occlusion.push(1.0);
            }
        }

        occlusion
    }

    /// Returns the calculated surface area of the mesh.
    pub fn surface_area(&self) -> f32 {
        let mut sum: f32 = 0.0;
        for tri in self.triangles.iter() {
            sum += tri.area(&self.positions);
        }
        sum
    }

    /// Shrinks mesh buffers to only use the necessary amount of memory.
    pub fn shrink_to_fit(&mut self) {
        self.triangles.shrink_to_fit();
        self.positions.shrink_to_fit();
        self.normals.shrink_to_fit();
        self.colors.shrink_to_fit();
        if let Some(mut uv1) = self.uv1.take() {
            uv1.shrink_to_fit();
            self.uv1 = Some(uv1);
        }
        if let Some(mut uv2) = self.uv2.take() {
            uv2.shrink_to_fit();
            self.uv1 = Some(uv2);
        }
    }

    /// Performs all existing optimization steps on the triangle mesh.
    pub fn optimize(&mut self, merge_distance: f32) {
        self.merge_by_distance(merge_distance);
        self.remove_degenerate();
        self.remove_unused();
        self.shrink_to_fit();
    }
}

impl Raycast for TriangleMesh {
    // TODO: method for raycasting many things at once and returning a list of results
    fn raycast(&self, params: RaycastParameters) -> Option<RaycastResult> {
        let mut shortest_depth: f32 = params.max_depth;
        let mut result = RaycastResult::default();

        // For all triangles
        for (idx, tri) in self.triangles.iter().enumerate() {
            // Perform a ray intersection
            let plane = tri.plane(&self.positions);

            // First, make sure this is shorter than our current collision depth
            // Also make sure it's not back-facing, if possible
            let depth = plane.signed_distance(params.origin);
            if (depth >= 0.0 || params.hit_backfaces) && depth < shortest_depth {
                // Project point onto the plane
                let projection = plane.ray_intersection(params.origin, params.direction);

                // TODO: better method for checking if ray direction is not hitting plane
                if projection.collided && (!projection.reversed || params.hit_backfaces) {
                    // Get barycentric coordinate of triangle
                    let coord = tri.barycentric(&self.positions, projection.intersection);
                    // Finally, check if the point is contained by the triangle
                    let contained = tri.contains_barycentric(coord);

                    if contained {
                        shortest_depth = depth;
                        result.point = projection.intersection;
                        result.normal = plane.xyz();
                        result.face_index = Some(idx);
                        result.barycentric = Some(coord);
                    }
                }
            }
        }

        // No collision, return nothing
        if shortest_depth == params.max_depth {
            return None;
        }

        result.depth = shortest_depth;
        Some(result)
    }
}

// UNIT TESTS //
#[cfg(test)]
mod tests {
    use std::f32;

    use super::{Edge, EdgeTriangles, TriangleMesh};
    use crate::math::raycast::RaycastParameters;
    use crate::{
        math::raycast::Raycast,
        mesh::trimesh::{Triangle, TriangleOperations},
    };
    use glam::{Vec3, vec3};

    const MAX_DIFFERENCE: f32 = 1e-7;

    /// Sanity test. Validate that EdgeMapEdge does not take extra memory.
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn type_sizes() {
        assert_eq!(8 * 2, std::mem::size_of::<Edge>(), "edge");
        assert_eq!(
            8 * 2,
            std::mem::size_of::<EdgeTriangles>(),
            "edge triangles"
        );
        assert_eq!(8 * 3, std::mem::size_of::<Triangle>(), "triangle");
    }

    #[test]
    fn test_calculate_normal() {
        struct TestFaceNormal {
            vertices: Vec<Vec3>,
            normal: Vec3,
        }

        let test_cases: Vec<TestFaceNormal> = vec![
            TestFaceNormal {
                vertices: vec![Vec3::NEG_X, Vec3::X, Vec3::Y],
                normal: Vec3::Z,
            },
            // Flipping the vertex order flips the face
            TestFaceNormal {
                vertices: vec![Vec3::Y, Vec3::X, Vec3::NEG_X],
                normal: Vec3::NEG_Z,
            },
            // A raised face still has the same normal
            TestFaceNormal {
                vertices: vec![
                    Vec3::new(-1.0, 0.0, 1.0),
                    Vec3::new(1.0, 0.0, 1.0),
                    Vec3::new(0.0, 1.0, 1.0),
                ],
                normal: Vec3::Z,
            },
            TestFaceNormal {
                vertices: vec![Vec3::NEG_X, Vec3::X, Vec3::Z],
                normal: Vec3::NEG_Y,
            },
            // Degenerate/non-manifold geometry simply returns an up vector
            TestFaceNormal {
                vertices: vec![Vec3::ZERO, Vec3::ZERO, Vec3::ZERO],
                normal: Vec3::Y,
            },
        ];

        for (idx, case) in test_cases.iter().enumerate() {
            let tri: Triangle = [0, 1, 2];
            let normal = tri.normal(&case.vertices);

            let dot = case.normal.dot(normal);
            assert!(
                dot >= 1.0 - MAX_DIFFERENCE,
                "Case {0}, expected {1} but got {2}, with dot product {3}",
                idx,
                case.normal,
                normal,
                dot
            );

            assert!(
                normal.length() > 1.0 - MAX_DIFFERENCE,
                "Case {idx}, {normal} isn't normalized ( >=1.0 )"
            );
            assert!(
                normal.length() < 1.0 + MAX_DIFFERENCE,
                "Case {idx}, {normal} isn't normalized ( <=1.0 )"
            );
        }
    }

    #[test]
    fn test_is_point_behind() {
        struct TestFaceIsPointBehind {
            tri: Vec<Vec3>,
            pt: Vec3,
            behind: bool,
        }
        let test_cases: Vec<TestFaceIsPointBehind> = vec![
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(0.0, 1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                ],
                pt: Vec3::Z,
                behind: true,
            },
            // Face normal is flipped, point should be in front of face
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(-1.0, -1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(0.0, 1.0, -1.0),
                ],
                pt: Vec3::Z,
                behind: false,
            },
            // Tested point is behind, but out of face's bounds
            TestFaceIsPointBehind {
                tri: vec![
                    Vec3::new(0.0, 1.0, -1.0),
                    Vec3::new(1.0, -1.0, -1.0),
                    Vec3::new(-1.0, -1.0, -1.0),
                ],
                pt: Vec3::new(10.0, 10.0, 10.0),
                behind: false,
            },
        ];

        for (idx, case) in test_cases.iter().enumerate() {
            let tri: Triangle = [0, 1, 2];
            let was_behind = tri.is_point_behind(&case.tri, case.pt);

            assert_eq!(
                was_behind,
                case.behind,
                "case {0}: triangle normal {1}",
                idx,
                tri.normal(&case.tri)
            );
        }
    }

    #[test]
    fn test_area() {
        let positions: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0),  // 0
            Vec3::new(1.0, 0.0, 0.0),  // 1
            Vec3::new(0.0, 1.0, 0.0),  // 2
            Vec3::new(-1.0, 0.0, 0.0), // 3
            Vec3::new(0.0, -1.0, 0.0), // 4
            Vec3::new(2.0, 0.0, 0.0),  // 5
            Vec3::new(0.0, 2.0, 0.0),  // 6
        ];
        let triangle_a: Triangle = [0, 2, 1];
        let triangle_b: Triangle = [0, 4, 3];
        let triangle_c: Triangle = [0, 6, 5];

        assert_eq!(0.5, triangle_a.area(&positions), "Triangle A");
        assert_eq!(0.5, triangle_b.area(&positions), "Triangle B");
        assert_eq!(2.0, triangle_c.area(&positions), "Triangle C");

        let triangles: Vec<Triangle> = vec![triangle_a, triangle_b, triangle_c];
        let mesh = TriangleMesh::new(triangles, positions, None, None);

        assert_eq!(3.0, mesh.surface_area(), "Mesh Surface Area");
    }

    #[test]
    fn test_remove_unused() {
        let positions: Vec<Vec3> = vec![
            Vec3::new(0.0, 0.0, 0.0), // 0
            Vec3::new(1.0, 0.0, 0.0), // 1
            Vec3::new(0.0, 1.0, 0.0), // 2
            Vec3::new(0.0, 1.0, 0.0), // 3
            Vec3::new(0.0, 0.0, 1.0), // 4
            Vec3::new(0.0, 0.0, 1.0), // 5
        ];
        let normals: Vec<Vec3> = positions.clone();
        let tris: Vec<Triangle> = vec![[1, 4, 3], [3, 4, 1]];
        let indices: Vec<usize> = vec![1, 4, 3, 3, 4, 1];

        let mut mesh =
            TriangleMesh::new(tris.clone(), positions.clone(), Some(normals.clone()), None);
        let mut index_mesh =
            TriangleMesh::from_indices(indices.clone(), positions.clone(), Some(normals.clone()));

        assert_eq!(
            tris, mesh.triangles,
            "created TriangleMesh has the same triangles"
        );
        assert_eq!(
            positions, mesh.positions,
            "created TriangleMesh has the same vertex positions"
        );
        assert_eq!(
            normals, mesh.normals,
            "created TriangleMesh has the same vertex normals"
        );
        assert_eq!(
            indices,
            mesh.indices(),
            "created TriangleMesh has the same indices"
        );

        assert_eq!(
            tris, index_mesh.triangles,
            "INDEXED TriangleMesh has the same triangles"
        );
        assert_eq!(
            positions, index_mesh.positions,
            "INDEXED TriangleMesh has the same vertex positions"
        );
        assert_eq!(
            normals, index_mesh.normals,
            "INDEXED TriangleMesh has the same vertex normals"
        );
        assert_eq!(
            indices,
            mesh.indices(),
            "created TriangleMesh has the same indices"
        );

        // Remove unused positions
        mesh.remove_unused();
        index_mesh.remove_unused();

        // Mesh vertex buffers shrank
        assert_eq!(3, mesh.positions.len(), "only 3 positions remain");
        assert_eq!(3, mesh.normals.len(), "only 3 normals remain");
        assert_eq!(3, index_mesh.positions.len(), "only 3 positions remain");
        assert_eq!(3, index_mesh.normals.len(), "only 3 normals remain");

        // Mesh should only contain our expected positions
        let positions_expected: Vec<Vec3> = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        assert_eq!(positions_expected, mesh.positions);
        assert_eq!(positions_expected, index_mesh.positions);

        // Indices should have been adjusted for resized buffers
        let indices_expected: Vec<usize> = vec![0, 2, 1, 1, 2, 0];
        assert_eq!(
            indices_expected,
            mesh.indices(),
            "should return expected vertex indices"
        );
        assert_eq!(
            indices_expected,
            index_mesh.indices(),
            "should return expected vertex indices"
        );
    }

    #[test]
    fn test_planar_decimation() {
        let positions: Vec<Vec3> = vec![
            vec3(1.0, 0.0, 0.0),
            vec3(-1.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            vec3(0.0, 0.0, -1.0),
        ];
        let triangles: Vec<Triangle> = vec![[0, 1, 2], [0, 3, 1]];
        let mut mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None, None);

        assert_eq!(
            1.0,
            triangles[0].orientation(&positions).signum(),
            "first triangle should be facing up"
        );
        assert_eq!(
            1.0,
            triangles[1].orientation(&positions).signum(),
            "second triangle should be facing up"
        );

        assert_eq!(
            0.0,
            mesh.face_angle(&triangles[0], &triangles[1]),
            "faces sharing same plane should have an angle of zero"
        );

        let edges = mesh.edge_map();
        assert_eq!(5, edges.len());

        mesh.decimate_planar(0.1, 10, 0);
        assert_eq!(0, mesh.triangles.len());
    }

    #[test]
    fn test_join() {
        let positions1: Vec<Vec3> = vec![Vec3::X, Vec3::Y, Vec3::Z];
        let positions2: Vec<Vec3> = vec![Vec3::NEG_X, Vec3::NEG_Y, Vec3::NEG_Z];

        let triangles = vec![[0, 1, 2]];
        let mut mesh1 = TriangleMesh::new(triangles.clone(), positions1.clone(), None, None);
        let mesh2 = TriangleMesh::new(triangles.clone(), positions2.clone(), None, None);

        assert_eq!(
            1,
            mesh1.triangles.len(),
            "mesh 1 should only have one triangle"
        );
        assert_eq!(
            1,
            mesh2.triangles.len(),
            "mesh 2 should only have one triangle"
        );
        assert_eq!(3, mesh1.positions.len(), "mesh 1 should have 3 vertices");
        assert_eq!(3, mesh2.positions.len(), "mesh 2 should have 3 vertices");

        mesh1.join(&mesh2);
        assert_eq!(
            6,
            mesh1.positions.len(),
            "joined mesh should have 6 vertices"
        );
        assert_eq!(
            2,
            mesh1.triangles.len(),
            "joined mesh should have 2 triangles"
        );

        assert_eq!(
            vec![
                Vec3::X,
                Vec3::Y,
                Vec3::Z,
                Vec3::NEG_X,
                Vec3::NEG_Y,
                Vec3::NEG_Z
            ],
            mesh1.positions
        );
        assert_eq!(vec![[0, 1, 2], [3, 4, 5]], mesh1.triangles);
    }

    #[test]
    fn test_merge_by_distance() {
        let positions: Vec<Vec3> = vec![
            vec3(1.0, 0.0, -1.0),
            vec3(-1.0, 0.0, -1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 1e-6, -1.0),
            vec3(-1.0, 1e-6, -1.0),
            vec3(0.0, 0.0, -1.0),
        ];
        let triangles = vec![[0, 1, 2], [3, 4, 5]];
        let mut mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None, None);

        mesh.merge_by_distance(1e-5);

        assert_eq!(2, mesh.triangles.len()); // Mesh still retains both triangles
        assert_eq!(
            vec![[0, 1, 2], [0, 1, 5]],
            mesh.triangles,
            "mesh uses only necessary points"
        );
        assert_eq!(6, mesh.positions.len()); // Mesh retained vertices but is not using them

        mesh.remove_unused();
        assert_eq!(4, mesh.positions.len(), "unused vertices should be removed");

        let mut mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None, None);
        mesh.optimize(1e-5);
        assert_eq!(
            vec![[0, 1, 2], [0, 1, 3]],
            mesh.triangles,
            "optimize only uses necessary points"
        );
        assert_eq!(4, mesh.positions.len(), "optimize should do all cleanup");
    }

    // TODO: edge map test using a manifold cube

    #[test]
    fn test_raycast_backface_triangles() {
        let positions: Vec<Vec3> = vec![
            vec3(1.0, 0.0, -1.0),
            vec3(-1.0, 0.0, -1.0),
            vec3(0.0, 0.0, 1.0),
        ];
        let triangles: Vec<Triangle> = vec![[0, 1, 2]];
        let mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None, None);

        let result = mesh
            .raycast(RaycastParameters::new(
                Vec3::Y,
                Vec3::NEG_Y,
                f32::INFINITY,
                false,
            ))
            .expect("raycast should hit directly");

        assert_eq!(
            Vec3::ZERO,
            result.point,
            "ray should should intersect at origin"
        );
        assert_eq!(result.normal, Vec3::Y, "normal should be facing the ray");
        assert_eq!(result.depth, 1.0, "depth should be 1");
        assert_eq!(
            0,
            result
                .face_index
                .expect("face index should be set for trimesh")
        );

        assert!(
            mesh.raycast(RaycastParameters::new(
                Vec3::NEG_Y,
                Vec3::Y,
                f32::INFINITY,
                false
            ))
            .is_none(),
            "raycast should miss backface"
        );
        assert!(
            mesh.raycast(RaycastParameters::new(
                Vec3::NEG_Y,
                Vec3::Y,
                f32::INFINITY,
                true
            ))
            .is_some(),
            "raycast should hit backface"
        );

        assert!(
            mesh.raycast(RaycastParameters::new(
                Vec3::new(5.0, 5.0, 5.0),
                Vec3::NEG_Y,
                f32::INFINITY,
                true
            ))
            .is_none(),
            "raycast should miss triangle during barycentric projection"
        );
    }

    #[test]
    fn test_raycast_offset() {
        let positions: Vec<Vec3> = vec![
            vec3(1.0, 1.0, -1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(0.0, 1.0, 1.0),
        ];
        let triangles: Vec<Triangle> = vec![[0, 1, 2]];
        let mesh = TriangleMesh::new(triangles.clone(), positions.clone(), None, None);

        let result = mesh
            .raycast(RaycastParameters::new(
                Vec3::new(0.1, 2.0, -0.5),
                Vec3::NEG_Y,
                f32::INFINITY,
                false,
            ))
            .expect("raycast should hit directly");

        assert_eq!(
            Vec3::new(0.1, 1.0, -0.5),
            result.point,
            "ray should should intersect at expected point"
        );
        assert_eq!(result.normal, Vec3::Y, "normal should be facing the ray");
        assert_eq!(result.depth, 1.0, "depth should be 1");
        assert_eq!(
            0,
            result
                .face_index
                .expect("face index should be set for trimesh")
        );

        // raycast that should completely miss
        let result = mesh.raycast(RaycastParameters::new(
            Vec3::new(0.1, 10.0, -0.5),
            Vec3::Y,
            f32::INFINITY,
            false,
        ));

        assert_eq!(None, result, "raycast should have missed");
    }

    #[test]
    fn test_raycast_layered() {
        // Should return nearest face index
        let positions_layered: Vec<Vec3> = vec![
            vec3(1.0, 0.0, -1.0),
            vec3(-1.0, 0.0, -1.0),
            vec3(0.0, 0.0, 1.0),
            vec3(1.0, 1.0, -1.0),
            vec3(-1.0, 1.0, -1.0),
            vec3(0.0, 1.0, 1.0),
        ];
        let triangles_layered: Vec<Triangle> = vec![[0, 1, 2], [3, 4, 5]];
        let mesh_layered = TriangleMesh::new(
            triangles_layered.clone(),
            positions_layered.clone(),
            None,
            None,
        );

        let result = mesh_layered
            .raycast(RaycastParameters::new(
                Vec3::Y * 3.0,
                Vec3::NEG_Y,
                f32::INFINITY,
                false,
            ))
            .expect("raycast should hit directly");

        assert_eq!(2.0, result.depth, "raycast should be 2 units from surface");
        assert_eq!(1, result.face_index.expect("face_index should exist"));
        assert_eq!(
            Vec3::Y,
            result.point,
            "raycast should intersect at (0, 1, 0)"
        );
    }
}
