use {
    crate::tri,
    criterion::{measurement::WallTime, BenchmarkGroup, Criterion},
    doublets::{
        mem::{Alloc, FileMapped},
        parts::LinkPart,
        split::{self, DataPart, IndexPart},
        unit, Doublets,
    },
    linksneo4j::{bench, connect, Benched, Client, Exclusive, Fork, Transaction, BACKGROUND_LINKS, LINK_COUNT},
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
    group.bench_function(id, |bencher| {
        bench!(|fork| as B {
            // Create additional links beyond background links to delete
            for _prepare in BACKGROUND_LINKS..BACKGROUND_LINKS + LINK_COUNT {
                let _ = fork.create_point();
            }
            // Delete the links we just created (in reverse order)
            for id in (BACKGROUND_LINKS + 1..=BACKGROUND_LINKS + LINK_COUNT).rev() {
                let _ = elapsed! {fork.delete(id)?};
            }
        })(bencher, &mut benched);
    });
}

pub fn delete_links(c: &mut Criterion) {
    let mut group = c.benchmark_group("Delete");
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
