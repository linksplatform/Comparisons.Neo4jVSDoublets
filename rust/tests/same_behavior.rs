//! Checks that every benchmarked store gives the same results for the
//! operations used by the benchmarks, and that undoing an operation (as the
//! benchmarks do after every iteration) restores the background links.
//!
//! Only point links and links with `source = target = 0` are used, as in the
//! benchmarks: the doublets 0.5.0 split store does not find links with
//! `source != target` by `[*, source, *]` or `[*, source, target]`
//! (see `experiments/split-store-bug`).
//!
//! The Neo4j test needs a running server (see `Neo4j::connect_from_env`):
//! `cargo test -- --include-ignored`.

use doublets::{
    data::{Flow, LinksConstants},
    Doublets, Link,
};
use linksneo4j::{
    Benched, DoubletsSplitNonVolatile, DoubletsSplitVolatile, DoubletsUnitedNonVolatile,
    DoubletsUnitedVolatile, Mode, Neo4j,
};

const BACKGROUND: usize = 20;
const LINKS: usize = 5;

/// Returns the links matching `query`, sorted by id.
fn query<B: Doublets<usize>>(store: &B, query: [usize; 3]) -> Vec<Link<usize>> {
    let mut links = Vec::new();
    store.each_by(query, |link| {
        links.push(link);
        Flow::Continue
    });
    links.sort_by_key(|link| link.index);
    links
}

fn all_links<B: Doublets<usize>>(store: &B) -> Vec<Link<usize>> {
    query(store, [store.constants().any; 3])
}

fn points(ids: impl IntoIterator<Item = usize>) -> Vec<Link<usize>> {
    ids.into_iter().map(Link::point).collect()
}

/// Runs the operations of the benchmarks inside `begin`/`commit` and checks
/// their results.
fn check<B: Benched + Doublets<usize>>(benched: &mut B) {
    let any = LinksConstants::<usize>::new().any;
    let mut store = benched.fork(BACKGROUND).unwrap();
    let background = points(1..=BACKGROUND);
    assert_eq!(all_links(&*store), background);

    // Create, undone by Delete.
    store.begin().unwrap();
    let created: Vec<_> = (0..LINKS).map(|_| store.create_point().unwrap()).collect();
    store.commit().unwrap();
    assert_eq!(
        created,
        (BACKGROUND + 1..=BACKGROUND + LINKS).collect::<Vec<_>>()
    );
    store.begin().unwrap();
    for id in (BACKGROUND + 1..=BACKGROUND + LINKS).rev() {
        store.delete(id).unwrap();
    }
    store.commit().unwrap();
    assert_eq!(all_links(&*store), background);

    // Delete, undone by Create.
    store.begin().unwrap();
    for id in (BACKGROUND - LINKS + 1..=BACKGROUND).rev() {
        store.delete(id).unwrap();
    }
    store.commit().unwrap();
    assert_eq!(all_links(&*store), points(1..=BACKGROUND - LINKS));
    store.begin().unwrap();
    for _ in 0..LINKS {
        store.create_point().unwrap();
    }
    store.commit().unwrap();
    assert_eq!(all_links(&*store), background);

    // Update, restored by the second update.
    store.begin().unwrap();
    store.update(BACKGROUND, 0, 0).unwrap();
    assert_eq!(
        store.get_link(BACKGROUND),
        Some(Link::new(BACKGROUND, 0, 0))
    );
    store.update(BACKGROUND, BACKGROUND, BACKGROUND).unwrap();
    store.commit().unwrap();
    assert_eq!(all_links(&*store), background);

    // The queries of the benchmarks.
    for id in 1..=BACKGROUND {
        let point = vec![Link::point(id)];
        assert_eq!(query(&*store, [id, any, any]), point);
        assert_eq!(query(&*store, [any, id, id]), point);
        assert_eq!(query(&*store, [any, id, any]), point);
        assert_eq!(query(&*store, [any, any, id]), point);
    }
    assert_eq!(store.count(), BACKGROUND);
}

#[test]
fn doublets_united_volatile() {
    check(&mut DoubletsUnitedVolatile::setup(()).unwrap());
}

#[test]
fn doublets_united_non_volatile() {
    let path = std::env::temp_dir().join("same_behavior_united.links");
    check(&mut DoubletsUnitedNonVolatile::setup(path.to_str().unwrap()).unwrap());
}

#[test]
fn doublets_split_volatile() {
    check(&mut DoubletsSplitVolatile::setup(()).unwrap());
}

#[test]
fn doublets_split_non_volatile() {
    let data = std::env::temp_dir().join("same_behavior_split_data.links");
    let index = std::env::temp_dir().join("same_behavior_split_index.links");
    check(
        &mut DoubletsSplitNonVolatile::setup((data.to_str().unwrap(), index.to_str().unwrap()))
            .unwrap(),
    );
}

/// Both modes share one database, so they run one after another in one test.
#[test]
#[ignore = "needs a running Neo4j server"]
fn neo4j() {
    for mode in [Mode::AutoCommit, Mode::Transaction] {
        check(&mut Neo4j::<usize>::setup(mode).unwrap());
    }
}
