//! Runs the exact operation sequences of the benchmark on the doublets 0.5.0
//! stores and checks after each one that every indexed query (outgoing,
//! incoming, concrete) agrees with a brute-force scan of all links.
use doublets::{
    data::{Flow, LinksConstants},
    mem::Global,
    split, unit, Doublets, Link,
};

const B: usize = 1000;
const N: usize = 100;

fn collect<S: Doublets<usize>>(store: &S, q: [usize; 3]) -> Vec<usize> {
    let mut ids = Vec::new();
    store.each_by(q, |l: Link<usize>| {
        ids.push(l.index);
        Flow::Continue
    });
    ids.sort();
    ids
}

fn verify<S: Doublets<usize>>(store: &S, after: &str) -> bool {
    let any = LinksConstants::<usize>::new().any;
    let all: Vec<Link<usize>> = {
        let mut v = Vec::new();
        store.each(|l| {
            v.push(l);
            Flow::Continue
        });
        v
    };
    let mut errors = 0;
    for id in 1..=B + N {
        let outgoing: Vec<_> = all.iter().filter(|l| l.source == id).map(|l| l.index).collect();
        let incoming: Vec<_> = all.iter().filter(|l| l.target == id).map(|l| l.index).collect();
        if collect(store, [any, id, any]) != outgoing || collect(store, [any, any, id]) != incoming {
            errors += 1;
        }
        for l in all.iter().filter(|l| l.source == id) {
            if collect(store, [any, l.source, l.target]) != vec![l.index] {
                errors += 1;
            }
        }
    }
    println!("  after {after:<40} links={} wrong queries={errors}", all.len());
    errors == 0
}

fn run<S: Doublets<usize>>(name: &str, mut store: S) {
    println!("{name}");
    for _ in 0..B {
        store.create_point().unwrap();
    }
    verify(&store, "creating the background");
    for _ in 0..N {
        store.create_point().unwrap();
    }
    verify(&store, "Create");
    for id in (B + 1..=B + N).rev() {
        store.delete(id).unwrap();
    }
    verify(&store, "undoing Create");
    for id in (B - N + 1..=B).rev() {
        store.delete(id).unwrap();
    }
    verify(&store, "Delete");
    for _ in 0..N {
        store.create_point().unwrap();
    }
    verify(&store, "undoing Delete");
    for id in B - N + 1..=B {
        store.update(id, 0, 0).unwrap();
    }
    verify(&store, "update(id, 0, 0)");
    for id in B - N + 1..=B {
        store.update(id, id, id).unwrap();
    }
    verify(&store, "update(id, id, id)");
    for id in B - N + 1..=B {
        store.update(id, 0, 0).unwrap();
        store.update(id, id, id).unwrap();
    }
    verify(&store, "Update (interleaved, as benchmarked)");
    store.delete_all().unwrap();
    println!("  delete_all ok, count={}", store.count());
}

fn main() {
    run("unit", unit::Store::<usize, _>::new(Global::new()).unwrap());
    run("split", split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap());
}
