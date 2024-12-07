use super::trimesh::TriangleMesh;
use crate::math::types::{ToVector3, Vec3};
use fast_surface_nets::SurfaceNetsBuffer;

/// Converts a `SurfaceNetsBuffer` to a `TriangleMesh`, returning `None` upon failure.
pub fn mesh_from_nets(
    nets: SurfaceNetsBuffer,
    scale: Vec3,
    translation: Vec3,
) -> Option<TriangleMesh> {
    if nets.indices.is_empty() {
        return None;
    }

    let indices = nets
        .indices
        .iter()
        .map(|idx| -> usize { *idx as usize })
        .collect::<Vec<usize>>();
    let positions = nets
        .positions
        .iter()
        .map(|pos| -> Vec3 {
            let p: Vec3 = pos.to_vector3();
            p * scale + translation
        })
        .collect::<Vec<Vec3>>();
    let normals = nets
        .normals
        .iter()
        .map(|norm| -> Vec3 {
            let n: Vec3 = norm.to_vector3();
            -n.normalize()
        })
        .collect::<Vec<Vec3>>();

    Some(TriangleMesh::from_indices(
        indices,
        positions,
        Some(normals),
    ))
}
