//! Minimal reproductions of wrong query results in the doublets 0.5.0 stores.
use doublets::{
    data::{Flow, LinksConstants},
    mem::Global,
    split, unit, Doublets, Link,
};

fn query<B: Doublets<usize>>(store: &B, q: [usize; 3]) -> Vec<Link<usize>> {
    let mut links = Vec::new();
    store.each_by(q, |l| {
        links.push(l);
        Flow::Continue
    });
    links.sort_by_key(|l| l.index);
    links
}

fn scenario<B: Doublets<usize>>(name: &str, store: &mut B, background: usize, prelude: bool) {
    let any = LinksConstants::<usize>::new().any;
    for _ in 0..background {
        store.create_point().unwrap();
    }
    if prelude {
        store.update(background, 0, 0).unwrap();
        store.update(background, background, background).unwrap();
    }
    store.update(3, 3, 7).unwrap();
    println!(
        "{name:<8} background={background:<3} prelude={prelude:<5} [3,*,*]={:?} [*,3,7]={:?} [*,3,*]={:?} [*,*,7]={:?} count={}",
        query(store, [3, any, any]),
        query(store, [any, 3, 7]),
        query(store, [any, 3, any]),
        query(store, [any, any, 7]),
        store.count_by([any, any, 7]),
    );
}

fn main() {
    for background in [3, 7, 8, 20] {
        for prelude in [false, true] {
            scenario("unit", &mut unit::Store::<usize, _>::new(Global::new()).unwrap(), background, prelude);
            scenario("split", &mut split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(), background, prelude);
        }
    }
}
