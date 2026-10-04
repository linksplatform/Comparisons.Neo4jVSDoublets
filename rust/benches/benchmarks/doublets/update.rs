//! # Doublets Update Benchmark
//!
//! Runs [`crate::benchmarks::update`] on all four Doublets stores.
//!
//! Detaches the link from the source and target index trees, writes the new
//! values and attaches it again (links with `0` as source or target are not
//! indexed).

use criterion::Criterion;

pub fn update_links(c: &mut Criterion) {
    crate::run_doublets!(c, "Update", crate::benchmarks::update);
}
