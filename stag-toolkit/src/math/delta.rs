/// Asserts that two numbers are within the given delta of each other.
pub fn assert_in_delta(expected: f32, actual: f32, delta: f32, descriptor: String) {
    assert!((expected - actual).abs() < delta, "{0}", descriptor);
}
