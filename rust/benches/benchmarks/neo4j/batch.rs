//! # Neo4j Batch Benchmarks
//!
//! The same work as [`crate::benchmarks`], but every iteration sends the `N`
//! operations to Neo4j as one list in one statement (two for Update), see
//! "Batches" in [`linksneo4j::neo4j_impl`].
//!
//! | Group           | Measured work of one iteration                                     | Undone after the iteration by     |
//! |-----------------|--------------------------------------------------------------------|-----------------------------------|
//! | `Create`        | `create_points_batch(N)`, which creates links `B + 1..=B + N`      | `delete_batch(B + 1..=B + N)`     |
//! | `Update`        | `update_batch` to `(0, 0)`, then `update_batch` back to `(id, id)` | nothing                           |
//! | `Delete`        | `delete_batch(B - N + 1..=B)`                                      | `create_points_batch(N)`          |
//! | `Each_*`        | `each_by_batch` with the `N` queries for the ids `1..=N`           | nothing (read only)               |
//!
//! Each All is already a single statement, so it has no batch variant.
//! Doublets has no batch calls: `N` operations are `N` function calls, which
//! is what the Doublets benchmarks measure.

use ::doublets::Link;
use criterion::{BenchmarkGroup, measurement::WallTime};
use linksneo4j::{Neo4j, background_links, benchmark_links};

use crate::benchmarks::{any, measure, read_only, visit};

pub fn create(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let (background, links) = (background_links(), benchmark_links());
    let created: Vec<_> = (background + 1..=background + links).collect();
    measure(
        group,
        id,
        neo4j,
        |store| store.create_points_batch(links),
        |store| store.delete_batch(&created),
    );
}

pub fn update(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let (background, links) = (background_links(), benchmark_links());
    let ids = background - links + 1..=background;
    let zeros: Vec<_> = ids.clone().map(|id| Link::new(id, 0, 0)).collect();
    let points: Vec<_> = ids.map(Link::point).collect();
    measure(
        group,
        id,
        neo4j,
        |store| {
            store.update_batch(&zeros)?;
            store.update_batch(&points)
        },
        read_only,
    );
}

pub fn delete(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let (background, links) = (background_links(), benchmark_links());
    let deleted: Vec<_> = (background - links + 1..=background).collect();
    measure(
        group,
        id,
        neo4j,
        |store| store.delete_batch(&deleted),
        |store| store.create_points_batch(links),
    );
}

/// Measures one `each_by_batch` with the queries `query(id)` for the ids
/// `1..=N`.
fn each_by(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    neo4j: &mut Neo4j<usize>,
    query: impl Fn(usize) -> [usize; 3],
) {
    let queries: Vec<_> = (1..=benchmark_links()).map(query).collect();
    measure(
        group,
        id,
        neo4j,
        |store| store.each_by_batch(&queries, |link| _ = visit(link)),
        read_only,
    );
}

pub fn each_identity(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let any = any();
    each_by(group, id, neo4j, |id| [id, any, any]);
}

pub fn each_concrete(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let any = any();
    each_by(group, id, neo4j, |id| [any, id, id]);
}

pub fn each_outgoing(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let any = any();
    each_by(group, id, neo4j, |id| [any, id, any]);
}

pub fn each_incoming(group: &mut BenchmarkGroup<WallTime>, id: &str, neo4j: &mut Neo4j<usize>) {
    let any = any();
    each_by(group, id, neo4j, |id| [any, any, id]);
}
