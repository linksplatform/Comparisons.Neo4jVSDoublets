//! # Each Outgoing Benchmark
//!
//! Measures the performance of querying links by source (outgoing edges).
//! This uses constraint `[*, source, *]` - finding all edges FROM a node.
//!
//! ## Common Interface Method
//!
//! ```rust,ignore
//! store.each_by([any, source, any], |link| Flow::Continue);
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Uses source index tree to find all links with given source
//! - Time complexity: O(log n + k) where k = matching links
//!
//! ### Neo4j
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $source
//! RETURN l.id, l.source, l.target
//! ```

use std::{
    alloc::Global,
    time::{Duration, Instant},
};

use criterion::{measurement::WallTime, BenchmarkGroup, Criterion};
use doublets::{
    data::{Flow, LinksConstants},
    mem::{Alloc, FileMapped},
    parts::LinkPart,
    split::{self, DataPart, IndexPart},
    unit, Doublets,
};
use linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction};

use crate::tri;

/// Runs the each_outgoing benchmark on a specific storage backend.
fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    let handler = |_| Flow::Continue;
    let any = LinksConstants::new().any;
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            use linksneo4j::BACKGROUND_LINKS;
            // The benchmarked operation: query by source (outgoing)
            // This calls the same interface method on both Doublets and Neo4j
            for index in 1..=BACKGROUND_LINKS {
                let _ = elapsed! {fork.each_by([any, index, any], handler)};
            }
        })(bencher, &mut benched);
    });
}

pub fn each_outgoing(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Outgoing");
    tri! {
        bench(&mut group, "Neo4j_NonTransaction", Exclusive::<Client<usize>>::setup(()).unwrap());
    }
    tri! {
        let client = connect().unwrap();
        bench(
            &mut group,
            "Neo4j_Transaction",
            Exclusive::<Transaction<'_, usize>>::setup(&client).unwrap(),
        );
    }
    tri! {
        bench(
            &mut group,
            "Doublets_United_Volatile",
            unit::Store::<usize, Alloc<LinkPart<_>, Global>>::setup(()).unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_United_NonVolatile",
            unit::Store::<usize, FileMapped<LinkPart<_>>>::setup("united.links").unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_Split_Volatile",
            split::Store::<usize, Alloc<DataPart<_>, _>, Alloc<IndexPart<_>, _>>::setup(()).unwrap()
        )
    }
    tri! {
        bench(
            &mut group,
            "Doublets_Split_NonVolatile",
            split::Store::<usize, FileMapped<_>, FileMapped<_>>::setup(("split_index.links", "split_data.links")).unwrap()
        )
    }
    group.finish();
}
