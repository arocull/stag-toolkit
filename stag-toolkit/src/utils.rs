use std::num::NonZero;
use std::thread::available_parallelism;

/// Returns the available parallelism, with a default non-zero value in unable to obtain.
pub fn thread_count(default_thread_count: usize) -> NonZero<usize> {
    #[cfg(debug_assertions)]
    assert_ne!(
        default_thread_count, 0,
        "default thread count cannot be zero"
    );
    available_parallelism()
        .unwrap_or_else(|_| NonZero::new(default_thread_count).expect("This should never fail"))
}

/// Returns the number of workers desired for the given workload size.
pub fn worker_count(workload_size: usize, default_thread_count: usize) -> NonZero<usize> {
    NonZero::new(
        (workload_size as f64 / thread_count(default_thread_count).get() as f64).ceil() as usize,
    )
    .expect("This should never fail")
}
