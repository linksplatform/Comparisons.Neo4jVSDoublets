#![feature(allocator_api)]

use benchmarks::{
    // Neo4j benchmarks
    neo4j_create_links, neo4j_delete_links, neo4j_each_all, neo4j_each_concrete,
    neo4j_each_identity, neo4j_each_incoming, neo4j_each_outgoing, neo4j_update_links,
    // Doublets benchmarks
    doublets_create_links, doublets_delete_links, doublets_each_all, doublets_each_concrete,
    doublets_each_identity, doublets_each_incoming, doublets_each_outgoing, doublets_update_links,
};
use criterion::{criterion_group, criterion_main};

mod benchmarks;

macro_rules! tri {
    ($($body:tt)*) => {
        let _ = (|| -> linksneo4j::Result<()> {
            Ok({ $($body)* })
        })().unwrap();
    };
}

pub(crate) use tri;

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

criterion_main!(neo4j_benches, doublets_benches);
