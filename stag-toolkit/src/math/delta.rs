use glam::Vec3;

/// Asserts that two numbers are within the given delta of each other.
pub fn assert_in_delta(expected: f32, actual: f32, delta: f32, descriptor: String) {
    assert!((expected - actual).abs() < delta, "{descriptor}");
}

pub fn assert_in_delta_vector(expected: Vec3, actual: Vec3, delta: f32, descriptor: &str) {
    assert!(
        (expected - actual).length() < delta,
        "{expected} != {actual}\t{descriptor}"
    );
}
