pub use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use godot::builtin::PackedColorArray;
use godot::builtin::PackedVector2Array;
pub use godot::builtin::Quaternion as QuatGodot;
pub use godot::builtin::Vector2 as Vec2Godot;
pub use godot::builtin::Vector3 as Vec3Godot;
use godot::builtin::{Basis, Color, Transform3D};
pub use godot::builtin::{PackedInt32Array, PackedVector3Array};
use mint::Vector3 as Vec3MintGeneral;

// VECTORS //

/// Mint-type 3D vector, used for ineroperability with other libraries.
pub type Vec3Mint = Vec3MintGeneral<f32>;

/// Implements 3D Vector conversion and ineroperability between math libraries.
pub trait ToVector3<T> {
    /// Converts a 3D vector from one type to another depending on the context.
    fn to_vector3(self) -> T;
}

// From Mint, to Glam
impl ToVector3<Vec3> for Vec3Mint {
    fn to_vector3(self) -> Vec3 {
        self.into()
    }
}
// From Glam, to Mint
impl ToVector3<Vec3Mint> for Vec3 {
    fn to_vector3(self) -> Vec3Mint {
        self.into()
    }
}
// From array, to Glam
impl ToVector3<Vec3> for [f32; 3] {
    fn to_vector3(self) -> Vec3 {
        Vec3::from_array(self)
    }
}
// From Glam, to array
impl ToVector3<[f32; 3]> for Vec3 {
    fn to_vector3(self) -> [f32; 3] {
        self.to_array()
    }
}
// From array, to Mint
impl ToVector3<Vec3Mint> for [f32; 3] {
    fn to_vector3(self) -> Vec3Mint {
        Vec3Mint::from(self)
    }
}
// From Mint, to array
impl ToVector3<[f32; 3]> for Vec3Mint {
    fn to_vector3(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}
// From Glam, to Godot
impl ToVector3<Vec3Godot> for Vec3 {
    fn to_vector3(self) -> Vec3Godot {
        Vec3Godot::new(self.x, self.y, self.z)
    }
}
// From Godot, to Glam
impl ToVector3<Vec3> for Vec3Godot {
    fn to_vector3(self) -> Vec3 {
        Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
// From Mint, to Godot
impl ToVector3<Vec3Godot> for Vec3Mint {
    fn to_vector3(self) -> Vec3Godot {
        Vec3Godot::from_array(self.into())
    }
}
// From Godot, to Mint
impl ToVector3<Vec3Mint> for Vec3Godot {
    fn to_vector3(self) -> Vec3Mint {
        self.to_array().into()
    }
}
// From Godot, to array
impl ToVector3<[f32; 3]> for Vec3Godot {
    fn to_vector3(self) -> [f32; 3] {
        self.to_array()
    }
}
// From array, to Godot
impl ToVector3<Vec3Godot> for [f32; 3] {
    fn to_vector3(self) -> Vec3Godot {
        Vec3Godot::from_array(self)
    }
}
/// Converts a series of Glam vectors into a PackedVector3Array for Godot.
impl ToVector3<PackedVector3Array> for Vec<Vec3> {
    fn to_vector3(self) -> PackedVector3Array {
        PackedVector3Array::from_iter(self.iter().map(|val| -> Vec3Godot { val.to_vector3() }))
    }
}

// 2D VECTORS //
/// Implements 2D Vector conversion and ineroperability between math libraries.
pub trait ToVector2<T> {
    /// Converts a 2D vector from one type to another depending on the context.
    fn to_vector2(self) -> T;
}
impl ToVector2<Vec2Godot> for Vec2 {
    fn to_vector2(self) -> Vec2Godot {
        Vec2Godot::new(self.x, self.y)
    }
}
impl ToVector2<Vec2> for Vec2Godot {
    fn to_vector2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}
impl ToVector2<PackedVector2Array> for Vec<Vec2> {
    fn to_vector2(self) -> PackedVector2Array {
        PackedVector2Array::from_iter(self.iter().map(|val| -> Vec2Godot { val.to_vector2() }))
    }
}

// COLORS //
/// Implements conversion between Vector4s and Colors.
pub trait ToColor<T> {
    /// Converts a vector from one type to another depending on the context.
    fn to_color(self) -> T;
}
// From Glam, to Godot
impl ToColor<Color> for Vec4 {
    fn to_color(self) -> Color {
        Color::from_rgba(self.x, self.y, self.z, self.w)
    }
}
impl ToColor<PackedColorArray> for Vec<Vec4> {
    fn to_color(self) -> PackedColorArray {
        PackedColorArray::from_iter(self.iter().map(|val| -> Color { val.to_color() }))
    }
}

// MATRICES //

/// Implements matrix conversion and ineroperability between math libraries.
pub trait ToTransform3D<T> {
    /// Converts a Transformation matrix from one type to another depending on the context.
    fn to_transform3d(self) -> T;
}
// From Glam, to Godot
impl ToTransform3D<Transform3D> for Mat4 {
    fn to_transform3d(self) -> Transform3D {
        let (scale, quat, loc) = self.to_scale_rotation_translation();

        Transform3D::new(
            Basis::from_quat(QuatGodot::new(quat.x, quat.y, quat.z, quat.w))
                .scaled(scale.to_vector3()),
            loc.to_vector3(),
        )
    }
}
// From Godot, to Glam
impl ToTransform3D<Mat4> for Transform3D {
    fn to_transform3d(self) -> Mat4 {
        let quat = self.basis.to_quat();
        Mat4::from_scale_rotation_translation(
            self.basis.scale().to_vector3(),
            Quat::from_xyzw(quat.x, quat.y, quat.z, quat.w),
            self.origin.to_vector3(),
        )
    }
}

/// Creates a PackedInt32Array from a vector of indices.
pub fn packed_index_array_usize(index_arr: Vec<usize>) -> PackedInt32Array {
    return PackedInt32Array::from_iter(index_arr.iter().map(|val| -> i32 { *val as i32 }));
}
/// Creates a PackedInt32Array from a vector of indices.
pub fn packed_index_array_u32(index_arr: Vec<u32>) -> PackedInt32Array {
    return PackedInt32Array::from_iter(index_arr.iter().map(|val| -> i32 { *val as i32 }));
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use godot::builtin::math::ApproxEq;

    use super::*;

    #[test]
    fn spatial_conversion() {
        let axis: Vec3 = Vec3::new(0.5, 0.7, 0.2).normalize();
        let angle: f32 = PI * 0.5;

        // Generate transforms in Godot and Glam separately
        let trangodot = Transform3D::new(
            Basis::from_axis_angle(axis.to_vector3(), angle),
            Vec3Godot::ZERO,
        );
        let tranglam = Mat4::from_axis_angle(axis, angle);

        // Convert the vectors to their counterparts
        let tranglam_godot: Transform3D = tranglam.to_transform3d();
        let trangodot_glam: Mat4 = trangodot.to_transform3d();

        // Assert that each transform is equal to its counterpart
        assert!(
            tranglam_godot.approx_eq(&trangodot),
            "Glam transform failed to convert to Godot"
        );
        assert!(
            trangodot_glam.abs_diff_eq(tranglam, std::f32::EPSILON),
            "Godot failed to convert to Glam"
        );
    }
}
