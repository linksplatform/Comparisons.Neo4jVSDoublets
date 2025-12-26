#![feature(allocator_api)]

use benchmarks::{
    create_links, delete_links, each_all, each_concrete, each_identity, each_incoming,
    each_outgoing, update_links,
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

criterion_group!(
    benches,
    create_links,
    delete_links,
    each_identity,
    each_concrete,
    each_outgoing,
    each_incoming,
    each_all,
    update_links
);
criterion_main!(benches);
