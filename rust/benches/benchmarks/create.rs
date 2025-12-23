use {
    crate::tri,
    criterion::{measurement::WallTime, BenchmarkGroup, Criterion},
    doublets::{
        data::LinkType,
        mem::{Alloc, FileMapped},
        parts::LinkPart,
        split::{self, DataPart, IndexPart},
        unit, Doublets,
    },
    linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, LINK_COUNT},
    std::{
        alloc::Global,
        time::{Duration, Instant},
    },
};

fn bench<T: LinkType, B: Benched + Doublets<T>>(
    group: &mut BenchmarkGroup<WallTime>,
    id: &str,
    mut benched: B,
) {
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            for _ in 0..LINK_COUNT {
                let _ = elapsed! {fork.create_point()?};
            }
        })(bencher, &mut benched);
    });
}

pub fn create_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Create");
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
