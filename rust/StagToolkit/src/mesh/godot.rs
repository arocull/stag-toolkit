use crate::math::sdf;
use crate::math::sdf::ShapeOperation;
use crate::math::types::*;
// use super::trimesh::TriangleMesh;
use godot::builtin::Array;
use godot::classes::csg_shape_3d::Operation;
use godot::classes::mesh::ArrayType;
use godot::classes::{CsgBox3D, CsgCylinder3D, CsgShape3D, CsgSphere3D, CsgTorus3D};
use godot::obj::IndexEnum;
use godot::prelude::*;

use super::trimesh::TriangleMesh;

// MESH DATA HANDLING //
/// A helper class for batch-handling mesh surface data within Godot Engine.
pub struct GodotSurfaceArrays {
    surface_arrays: Array<Variant>,
}
impl Default for GodotSurfaceArrays {
    fn default() -> Self {
        Self::new()
    }
}

impl GodotSurfaceArrays {
    /// Initializes a new set of mesh surface arrays.
    /// Vertices and indices are not set initially,
    /// as they will always have to be set manually.
    pub fn new() -> Self {
        let mut sa = Array::new();

        sa.resize(
            ArrayType::MAX.to_index(),
            &Array::<Variant>::new().to_variant(),
        );

        // Bind vertex data
        // sa.set(ArrayType::VERTEX.to_index(), &Variant::nil()); // Overridden anyway
        sa.set(ArrayType::NORMAL.to_index(), &Variant::nil());
        sa.set(ArrayType::TANGENT.to_index(), &Variant::nil());

        // Bind masking data
        sa.set(ArrayType::COLOR.to_index(), &Variant::nil());

        // Bind UV projections
        sa.set(ArrayType::TEX_UV.to_index(), &Variant::nil());
        sa.set(ArrayType::TEX_UV2.to_index(), &Variant::nil());

        // Bind custom arrays
        sa.set(ArrayType::CUSTOM0.to_index(), &Variant::nil());
        sa.set(ArrayType::CUSTOM1.to_index(), &Variant::nil());
        sa.set(ArrayType::CUSTOM2.to_index(), &Variant::nil());
        sa.set(ArrayType::CUSTOM3.to_index(), &Variant::nil());

        // Bind skeleton
        sa.set(ArrayType::BONES.to_index(), &Variant::nil());
        sa.set(ArrayType::WEIGHTS.to_index(), &Variant::nil());

        // FINALLY, bind indices (actually don't bother since we'll be overriding them anyway)
        // sa.set(ArrayType::INDEX.to_index(), &Variant::nil()); // Overridden anyway

        Self { surface_arrays: sa }
    }

    /// Creates a corresponding GodotSurfaceArrays set from a TriangleMesh.
    pub fn from_trimesh(mesh: &TriangleMesh) -> Self {
        let mut surface = Self::new();

        surface.set_vertices(mesh.positions.clone().to_vector3());
        surface.set_normals(mesh.normals.clone().to_vector3());
        surface.set_indices(packed_index_array_usize(mesh.indices()));

        surface
    }

    /// Internally sets a SurfaceArray value to the given variant.
    fn set_internal(&mut self, arrtype: ArrayType, value: Variant) {
        self.surface_arrays.set(arrtype.to_index(), &value);
    }

    /// Sets the indices buffer
    pub fn set_indices(&mut self, value: PackedInt32Array) {
        self.set_internal(ArrayType::INDEX, value.to_variant());
    }
    /// Sets the vertex position buffer
    pub fn set_vertices(&mut self, value: PackedVector3Array) {
        self.set_internal(ArrayType::VERTEX, value.to_variant());
    }
    /// Sets the vertex normal buffer
    pub fn set_normals(&mut self, value: PackedVector3Array) {
        self.set_internal(ArrayType::NORMAL, value.to_variant());
    }
    /// Sets the vertex tangent buffer
    pub fn set_tangents(&mut self, value: PackedVector3Array) {
        self.set_internal(ArrayType::TANGENT, value.to_variant());
    }
    /// Sets the vertex color buffer
    pub fn set_colors(&mut self, value: PackedColorArray) {
        self.set_internal(ArrayType::COLOR, value.to_variant());
    }
    /// Sets the vertex UV1 buffer
    pub fn set_uv1(&mut self, value: PackedVector2Array) {
        self.set_internal(ArrayType::TEX_UV, value.to_variant());
    }
    /// Sets the vertex UV2 buffer
    pub fn set_uv2(&mut self, value: PackedVector2Array) {
        self.set_internal(ArrayType::TEX_UV2, value.to_variant());
    }

    /// Returns a copy of the surface arrays, for passing to Godot.
    pub fn get_surface_arrays(&self) -> Array<Variant> {
        self.surface_arrays.clone()
    }
}

/// A collection of Signed Distance Field shapes for sampling.
#[derive(Clone)]
pub struct GodotWhitebox {
    /// List of shapes contained by the whitebox.
    shapes: Vec<sdf::Shape>,
    /// The default edge radius for a shape, to use when not pre-defined.
    pub default_edge_radius: f32,
}
impl Default for GodotWhitebox {
    fn default() -> Self {
        Self::new()
    }
}

impl GodotWhitebox {
    /// Generates a new, empty whitebox.
    pub fn new() -> Self {
        Self {
            shapes: vec![],
            default_edge_radius: 0.0,
        }
    }

    /// Samples the whitebox shape list at the given position, returning the distance to its surface.
    pub fn sample_at(&self, point: Vec3Godot) -> f32 {
        sdf::sample_shape_list(&self.shapes, point.to_vector3())
    }

    /// Clears the shape list.
    pub fn clear(&mut self) {
        self.shapes.clear();
    }
    /// Returns the shape list.
    pub fn get_shapes(&self) -> &Vec<sdf::Shape> {
        self.shapes.as_ref()
    }
    /// Returns the number of shapes.
    pub fn get_shape_count(&self) -> usize {
        self.shapes.len()
    }
    /// Calculates the Axis-Aligned Bounding Box for the whitebox.
    pub fn get_aabb(&self) -> Aabb {
        let mut aabb = Aabb::new(Vec3Godot::ZERO, Vec3Godot::ZERO);

        // If we have no shapes, return nothing
        if self.shapes.is_empty() {
            return aabb;
        }

        // Create an iterator
        for shape in self.shapes.iter() {
            // Only include Unions in AABB, as all other operations take away
            if shape.operation != ShapeOperation::Union {
                continue;
            }

            let (min_bound, max_bound) = shape.relative_bounds();
            let shape_aabb = shape.transform().to_transform3d()
                * Aabb::new(min_bound.to_vector3(), (max_bound - min_bound).to_vector3());
            aabb = aabb.merge(shape_aabb);
        }

        aabb
    }

    /// Serializes CSG geometry into a whitebox. Temporarily shows the parent node in case it is hidden.
    pub fn serialize_from(&mut self, mut node: Gd<Node3D>) {
        let was_visible = node.is_visible();
        node.set_visible(true);
        self.serialize_walk(node.clone(), node.clone().upcast::<Node>());
        node.set_visible(was_visible);
    }

    /// Walks a single step in the node tree, serializing the current shape
    fn serialize_walk(&mut self, parent: Gd<Node3D>, node: Gd<Node>) {
        for child in node.get_children().iter_shared() {
            self.serialize_walk(parent.clone(), child);
        }

        let mut transform: Transform3D; // relative transform of node
        let op: ShapeOperation; // CSG operation of node

        // First, do generic cast to get basic Node3D properties
        if let Ok(shape) = node.clone().try_cast::<CsgShape3D>() {
            // If the shape is hidden, don't serialize it!
            if !shape.is_visible_in_tree() {
                return;
            }

            // Get relative transform
            // TODO: use a recursive transform tree until we get to the top-level IslandBuilder,
            // instead of just using the immediate parent
            transform =
                parent.get_global_transform().affine_inverse() * shape.get_global_transform();

            // Get node's CSG operation
            op = csg_operation(shape.get_operation());
        } else {
            // This isn't a valid node!
            return;
        }

        // Then, cast to each type of CSG class
        if let Ok(csg) = node.clone().try_cast::<CsgBox3D>() {
            // Since we have a box, we can pull out the scale
            let mut scale = transform.basis.get_scale();
            // ...and unscale the transform!
            transform = transform.scaled_local(Vec3Godot::ONE / scale);
            // Also, don't forget to factor in the original CSG box scale on top
            scale *= csg.get_size();

            // Finally, store shape
            self.shapes.push(sdf::Shape::rounded_box(
                transform.to_transform3d(),
                scale.to_vector3(),
                self.fetch_edge_radius(csg.upcast::<Node>()),
                op,
            ));
        } else if let Ok(csg) = node.clone().try_cast::<CsgSphere3D>() {
            self.shapes.push(sdf::Shape::sphere(
                transform.to_transform3d(),
                csg.get_radius(),
                op,
            ));
        } else if let Ok(csg) = node.clone().try_cast::<CsgCylinder3D>() {
            let mut scale = transform.basis.get_scale();
            transform = transform.scaled_local(Vec3Godot::ONE / Vec3Godot::new(1.0, scale.y, 1.0));
            scale.y *= csg.get_height();

            self.shapes.push(sdf::Shape::rounded_cylinder(
                transform.to_transform3d(),
                scale.y,
                csg.get_radius(),
                self.fetch_edge_radius(csg.upcast::<Node>()),
                op,
            ));
        } else if let Ok(csg) = node.clone().try_cast::<CsgTorus3D>() {
            let thickness = (csg.get_outer_radius() - csg.get_inner_radius()).abs() * 0.5;
            self.shapes.push(sdf::Shape::torus(
                transform.to_transform3d(),
                thickness,
                (csg.get_outer_radius() - thickness).abs(),
                op,
            ));
        }
    }

    /// Fetches the given metadata float from a node, or returns a default
    fn fetch_meta(node: Gd<Node>, meta_name: StringName, default: f32) -> f32 {
        if node.has_meta(&meta_name) {
            return node.get_meta(&meta_name).to();
        }
        default
    }
    /// Fetches the edge radius of a whitebox node
    fn fetch_edge_radius(&self, node: Gd<Node>) -> f32 {
        Self::fetch_meta(node, "edge_radius".into(), self.default_edge_radius)
    }
}

// HELPER FUNCTIONS

fn csg_operation(gd_op: Operation) -> ShapeOperation {
    match gd_op {
        Operation::INTERSECTION => ShapeOperation::Intersection,
        Operation::SUBTRACTION => ShapeOperation::Subtraction,
        _ => ShapeOperation::Union,
    }
}
