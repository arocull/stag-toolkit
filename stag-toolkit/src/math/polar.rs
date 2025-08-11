use glam::Vec3;

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(C)]
pub struct Polar {
    /// AKA latitude or yaw.
    pub azimuth: f32,
    /// AKA longitude or pitch.
    pub elevation: f32,
}

impl Polar {
    pub fn from_cartesian(direction: Vec3) -> Self {
        // https://stackoverflow.com/questions/10868135/cartesian-to-polar-3d-coordinates
        // TODO: allow specifying cartesian axii
        #[cfg(debug_assertions)]
        assert!(direction.is_normalized(), "direction vector should be normalized");

        let elevation = direction.x.acos() * (if direction.y < 0.0 { -1.0 } else { 1.0});
        let azimuth = direction.z.acos();

        Self {
            azimuth,
            elevation,
        }
    }

    pub fn to_cartesian(&self, radius: f32) -> Vec3 {
        // https://stackoverflow.com/questions/10868135/cartesian-to-polar-3d-coordinates
        Vec3::new(
            self.azimuth.sin() * radius,
            self.azimuth.sin() * self.elevation.sin() * radius,
            self.azimuth.cos() * radius,
        )
    }
}
