use std::num::NonZero;
use std::thread::available_parallelism;

pub fn thread_count() -> NonZero<usize> {
    available_parallelism().unwrap_or_else(|_| NonZero::new(16).expect("This should never fail"))
}
