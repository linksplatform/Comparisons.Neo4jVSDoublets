//! # Benchmark Implementations
//!
//! The operations below are written once and run unchanged on every backend,
//! so Neo4j and Doublets execute exactly the same sequence of
//! [`Doublets`] calls.
//!
//! ## Module Structure
//!
//! - **[`neo4j`]** - runs the operations on Neo4j (both transaction modes),
//!   and the same work as batches
//! - **[`doublets`]** - runs the operations on the four Doublets stores
//!
//! ## Benchmarked Operations
//!
//! `B` is [`background_links`] and `N` is [`benchmark_links`]. Before every
//! iteration the store contains the point links `1..=B`.
//!
//! | Group           | Measured work of one iteration                                      | Undone after the iteration by                 |
//! |-----------------|---------------------------------------------------------------------|-----------------------------------------------|
//! | `Create`        | `N` × `create_point()`, which creates links `B + 1..=B + N`         | deleting links `B + N` to `B + 1`             |
//! | `Update`        | `N` × (`update(id, 0, 0)` then `update(id, id, id)`) = `2N` updates | nothing (the second update restores the link) |
//! | `Delete`        | `N` × `delete(id)`, from link `B` down to link `B - N + 1`          | `N` × `create_point()`                        |
//! | `Each_All`      | 1 × `each(...)`, which visits all `B` links                         | nothing (read only)                           |
//! | `Each_Identity` | `N` × `each_by([id, *, *])`                                         | nothing (read only)                           |
//! | `Each_Concrete` | `N` × `each_by([*, id, id])`                                        | nothing (read only)                           |
//! | `Each_Outgoing` | `N` × `each_by([*, id, *])`                                         | nothing (read only)                           |
//! | `Each_Incoming` | `N` × `each_by([*, *, id])`                                         | nothing (read only)                           |
//!
//! The measured time of an iteration starts before [`Benched::begin`] and ends
//! after [`Benched::commit`], so for Neo4j in transaction mode it includes
//! starting and committing the transaction.

use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use ::doublets::{
    Doublets, Link,
    data::{Flow, LinksConstants},
};
use criterion::{BenchmarkGroup, Criterion, SamplingMode, measurement::WallTime};
use linksneo4j::{Benched, Result, background_links, benchmark_links};

pub mod doublets;
pub mod neo4j;

// Re-export all Neo4j benchmarks with neo4j_ prefix
pub use neo4j::create_links as neo4j_create_links;
pub use neo4j::delete_links as neo4j_delete_links;
pub use neo4j::each_all as neo4j_each_all;
pub use neo4j::each_concrete as neo4j_each_concrete;
pub use neo4j::each_identity as neo4j_each_identity;
pub use neo4j::each_incoming as neo4j_each_incoming;
pub use neo4j::each_outgoing as neo4j_each_outgoing;
pub use neo4j::update_links as neo4j_update_links;

// Re-export all Doublets benchmarks with doublets_ prefix
pub use self::doublets::create_links as doublets_create_links;
pub use self::doublets::delete_links as doublets_delete_links;
pub use self::doublets::each_all as doublets_each_all;
pub use self::doublets::each_concrete as doublets_each_concrete;
pub use self::doublets::each_identity as doublets_each_identity;
pub use self::doublets::each_incoming as doublets_each_incoming;
pub use self::doublets::each_outgoing as doublets_each_outgoing;
pub use self::doublets::update_links as doublets_update_links;

/// Signature shared by all operations, so they can be passed around.
pub type Operation<B> = fn(&mut BenchmarkGroup<WallTime>, &str, &mut B);

/// Creates a benchmark group for Neo4j.
///
/// One Neo4j iteration takes milliseconds, so the minimum number of samples
/// (10) with one or more iterations each already gives stable results, and
/// flat sampling avoids the many extra iterations of linear sampling.
pub fn neo4j_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group
        .sample_size(10)
        .sampling_mode(SamplingMode::Flat)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    group
}

/// Creates a benchmark group for Doublets.
///
/// One Doublets iteration takes microseconds, so Criterion's default number
/// of samples (100) is used to measure it precisely.
pub fn doublets_group<'a>(c: &'a mut Criterion, name: &str) -> BenchmarkGroup<'a, WallTime> {
    let mut group = c.benchmark_group(name);
    group
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    group
}

/// Measures `operation` on a store that contains [`background_links`] point
/// links.
///
/// The background links are created once, when Criterion first runs the
/// benchmark (so benchmarks skipped by a filter create no links). After every
/// iteration `undo` puts the store back into the same state, so every
/// iteration starts from the same links. Neither of them is measured.
pub(crate) fn measure<B: Benched>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
    mut operation: impl FnMut(&mut B) -> Result<()>,
    mut undo: impl FnMut(&mut B) -> Result<()>,
) {
    let mut benched = Some(benched);
    let mut store = None;
    let mut iteration = |store: &mut B| -> Result<Duration> {
        let started = Instant::now();
        store.begin()?;
        operation(store)?;
        store.commit()?;
        let elapsed = started.elapsed();

        store.begin()?;
        undo(store)?;
        store.commit()?;
        Ok(elapsed)
    };
    group.bench_function(id, |bencher| {
        let store = store.get_or_insert_with(|| {
            let benched = benched.take().expect("the store is forked only once");
            benched
                .fork(background_links())
                .expect("failed to create the background links")
        });
        bencher.iter_custom(|iterations| {
            (0..iterations)
                .map(|_| iteration(store).expect("benchmark iteration failed"))
                .sum()
        })
    });
}

/// Nothing to undo.
pub(crate) fn read_only<B>(_: &mut B) -> Result<()> {
    Ok(())
}

/// Consumes every found link, so the compiler cannot skip reading it.
pub(crate) fn visit(link: Link<usize>) -> Flow {
    black_box(link);
    Flow::Continue
}

pub(crate) fn any() -> usize {
    LinksConstants::<usize>::new().any
}

pub fn create<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let (background, links) = (background_links(), benchmark_links());
    measure(
        group,
        id,
        benched,
        |store| {
            for _ in 0..links {
                store.create_point()?;
            }
            Ok(())
        },
        |store| {
            for id in (background + 1..=background + links).rev() {
                store.delete(id)?;
            }
            Ok(())
        },
    );
}

pub fn update<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let (background, links) = (background_links(), benchmark_links());
    measure(
        group,
        id,
        benched,
        |store| {
            for id in background - links + 1..=background {
                store.update(id, 0, 0)?;
                store.update(id, id, id)?;
            }
            Ok(())
        },
        read_only,
    );
}

pub fn delete<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let (background, links) = (background_links(), benchmark_links());
    measure(
        group,
        id,
        benched,
        |store| {
            for id in (background - links + 1..=background).rev() {
                store.delete(id)?;
            }
            Ok(())
        },
        |store| {
            for _ in 0..links {
                store.create_point()?;
            }
            Ok(())
        },
    );
}

pub fn each_all<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    measure(
        group,
        id,
        benched,
        |store| {
            store.each(visit);
            Ok(())
        },
        read_only,
    );
}

/// Measures `N` queries `each_by(query(id))` for the ids `1..=N`.
fn each_by<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
    query: impl Fn(usize) -> [usize; 3],
) {
    let links = benchmark_links();
    measure(
        group,
        id,
        benched,
        |store| {
            for id in 1..=links {
                store.each_by(query(id), visit);
            }
            Ok(())
        },
        read_only,
    );
}

pub fn each_identity<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let any = any();
    each_by(group, id, benched, |id| [id, any, any]);
}

pub fn each_concrete<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let any = any();
    each_by(group, id, benched, |id| [any, id, id]);
}

pub fn each_outgoing<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let any = any();
    each_by(group, id, benched, |id| [any, id, any]);
}

pub fn each_incoming<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    benched: &mut B,
) {
    let any = any();
    each_by(group, id, benched, |id| [any, any, id]);
}
