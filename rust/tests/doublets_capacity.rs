//! Checks the capacity limit of the doublets 0.5.0 stores described in
//! `linksneo4j::doublets_impl` (section "Capacity").
//!
//! If these tests fail after a doublets update, the limit may be gone and
//! larger numbers of background links can be benchmarked.

use doublets::Doublets;
use linksneo4j::{Benched, DoubletsUnitedVolatile, doublets_impl::MAX_LINKS};

fn store_with_max_links() -> DoubletsUnitedVolatile {
    let mut store: DoubletsUnitedVolatile = Benched::setup(()).unwrap();
    for _ in 0..MAX_LINKS {
        store.create_point().unwrap();
    }
    store
}

#[test]
fn store_holds_max_links() {
    assert_eq!(store_with_max_links().count(), MAX_LINKS);
}

#[test]
#[should_panic(expected = "Data part should be in data memory")]
fn store_cannot_hold_one_more_link() {
    store_with_max_links().create_point().unwrap();
}

#[test]
fn fork_reports_too_many_links() {
    let mut store: DoubletsUnitedVolatile = Benched::setup(()).unwrap();
    let error = store.fork(MAX_LINKS).err().expect("the fork must fail");
    assert!(error.to_string().contains("at most"), "{error}");
}
