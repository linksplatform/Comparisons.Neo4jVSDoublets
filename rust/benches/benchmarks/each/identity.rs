use {
    crate::tri,
    criterion::{measurement::WallTime, BenchmarkGroup, Criterion},
    doublets::{
        data::{Flow, LinksConstants},
        mem::{Alloc, FileMapped},
        parts::LinkPart,
        split::{self, DataPart, IndexPart},
        unit, Doublets,
    },
    linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction},
    std::{
        alloc::Global,
        time::{Duration, Instant},
    },
};

fn bench<B: Benched + Doublets<usize>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    let handler = |_| Flow::Continue;
    let any = LinksConstants::new().any;
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            for index in 1..=1_000 {
                elapsed! {fork.each_by([index, any, any], handler)};
            }
            for index in 1_001..=2_000 {
                elapsed! {fork.each_by([index, any, any], handler)};
            }
            for index in 2_001..=BACKGROUND_LINKS {
                elapsed! {fork.each_by([index, any, any], handler)};
            }
        })(bencher, &mut benched);
    });
}

pub fn each_identity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Each_Identity");
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
