//! # Each Concrete Benchmark
//!
//! Measures the performance of querying links by BOTH source AND target.
//! This uses constraint `[*, source, target]` - a composite index lookup.
//!
//! ## Common Interface Method
//!
//! ```rust,ignore
//! store.each_by([any, source, target], |link| Flow::Continue);
//! ```
//!
//! ## How Each Database Implements It
//!
//! ### Doublets
//! - Uses source OR target index tree to find candidates
//! - Filters by the other field
//! - Time complexity: O(log n) for tree traversal
//!
//! ### Neo4j
//! ```cypher
//! MATCH (l:Link) WHERE l.source = $s AND l.target = $t
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

/// Runs the each_concrete benchmark on a specific storage backend.
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
            // The benchmarked operation: query by source AND target
            // This calls the same interface method on both Doublets and Neo4j
            for index in 1..=BACKGROUND_LINKS {
                elapsed! {fork.each_by([any, index, index], handler)};
            }
        })(bencher, &mut benched);
    });
}

pub fn each_concrete(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Concrete");
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
