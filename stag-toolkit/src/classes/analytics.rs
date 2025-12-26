use glam::{IVec2, IVec3, IVec4, Mat4, Vec2, Vec3, Vec4};
use godot::{
    classes::{Engine, Time},
    prelude::*,
};
use std::collections::HashMap;

pub enum GenericData {
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Vector2i(IVec2),
    Vector3i(IVec3),
    Vector4i(IVec4),
    Vector2(Vec2),
    Vector3(Vec3),
    Vector4(Vec4),
    Quaternion(Vec4),
    Transform(Mat4),
}

/// Can be used to store various analytics data before being packaged up and sent to a given endpoint.
///
/// @experimental: Still debating implementation.
#[derive(GodotClass)]
#[class(init,base=Node,tool)]
pub struct StagAnalytics {
    /// Current time offset, in milliseconds, from start of recording.
    #[export(range=(0.0,50000.0,or_greater,suffix="ms"))]
    #[init(val = 0)]
    pub time_offset: i64,

    /// Duration of time between data points when recording is active.
    #[export_group(name = "Time", prefix = "time_")]
    #[export(range=(1.0, 5000.0,or_greater,suffix="ms"))]
    #[init(val = 2000)]
    pub time_resolution: u32,

    /// Whether to enable Zstd compression when pulling data as a byte array.
    #[export]
    #[export_group(name = "Compression", prefix = "compression_")]
    #[init(val = false)]
    pub compression_enabled: bool,

    /// Zstd compression level to use when compressing data into a byte array.
    #[export(range=(-7.0, 22.0))]
    #[export_group(name = "Compression", prefix = "compression_")]
    #[init(val = 0)]
    pub compression_level: i8,

    #[init(val = false)]
    recording: bool,

    #[init(val = 0)]
    record_step: u32,

    #[init(val=vec!())]
    data: Vec<HashMap<u32, GenericData>>,
}

#[godot_api]
impl INode for StagAnalytics {}

#[godot_api]
impl StagAnalytics {
    #[func]
    fn record_vector3(&mut self, key: GString, val: Vector3) {}

    #[func]
    fn get_value(&self, key: GString) {}
}
