use benchmarks::{
    // Doublets benchmarks
    doublets_create_links,
    doublets_delete_links,
    doublets_each_all,
    doublets_each_concrete,
    doublets_each_identity,
    doublets_each_incoming,
    doublets_each_outgoing,
    doublets_update_links,
    // Neo4j benchmarks
    neo4j_create_links,
    neo4j_delete_links,
    neo4j_each_all,
    neo4j_each_concrete,
    neo4j_each_identity,
    neo4j_each_incoming,
    neo4j_each_outgoing,
    neo4j_update_links,
};
use criterion::{Criterion, criterion_group};

mod benchmarks;

// Neo4j benchmarks group
criterion_group!(
    neo4j_benches,
    neo4j_create_links,
    neo4j_delete_links,
    neo4j_each_identity,
    neo4j_each_concrete,
    neo4j_each_outgoing,
    neo4j_each_incoming,
    neo4j_each_all,
    neo4j_update_links
);

// Doublets benchmarks group
criterion_group!(
    doublets_benches,
    doublets_create_links,
    doublets_delete_links,
    doublets_each_identity,
    doublets_each_concrete,
    doublets_each_outgoing,
    doublets_each_incoming,
    doublets_each_all,
    doublets_update_links
);

/// Runs the benchmarks of both databases, or only of the one named by the
/// `BENCHMARK_BACKEND` environment variable (`neo4j` or `doublets`), so that
/// they can run in parallel CI jobs and the Doublets job needs no Neo4j server.
fn main() {
    let backend = std::env::var("BENCHMARK_BACKEND").unwrap_or_default();
    match backend.as_str() {
        "" => {
            neo4j_benches();
            doublets_benches();
        }
        "neo4j" => neo4j_benches(),
        "doublets" => doublets_benches(),
        _ => panic!("BENCHMARK_BACKEND must be `neo4j` or `doublets`, got `{backend}`"),
    }
    Criterion::default().configure_from_args().final_summary();
}
