//! Which links does the doublets 0.5.0 split store fail to find?
use doublets::{
    data::{Flow, LinksConstants},
    mem::Global,
    split, unit, Doublets,
};

fn count<B: Doublets<usize>>(store: &B, q: [usize; 3]) -> usize {
    let mut n = 0;
    store.each_by(q, |_| {
        n += 1;
        Flow::Continue
    });
    n
}

fn check<B: Doublets<usize>>(name: &str, mut store: B) {
    let any = LinksConstants::<usize>::new().any;
    for _ in 0..10 {
        store.create_point().unwrap();
    }
    // points: 1..=10. Modify some links:
    store.update(3, 3, 7).unwrap(); // target > source
    store.update(8, 8, 2).unwrap(); // target < source
    store.update(9, 5, 5).unwrap(); // not a point, same source and target
    let mut report = Vec::new();
    for (label, q, expected) in [
        ("point 5 outgoing [*,5,*]", [any, 5, any], 2),
        ("point 5 concrete [*,5,5]", [any, 5, 5], 2),
        ("point 5 incoming [*,*,5]", [any, any, 5], 2),
        ("point 1 outgoing [*,1,*]", [any, 1, any], 1),
        ("point 10 outgoing [*,10,*]", [any, 10, any], 1),
        ("3->7 outgoing [*,3,*]", [any, 3, any], 1),
        ("3->7 concrete [*,3,7]", [any, 3, 7], 1),
        ("8->2 outgoing [*,8,*]", [any, 8, any], 1),
        ("8->2 concrete [*,8,2]", [any, 8, 2], 1),
        ("incoming [*,*,7]", [any, any, 7], 2),
        ("incoming [*,*,2]", [any, any, 2], 2),
    ] {
        let found = count(&store, q);
        let fail = if found == expected { "" } else { "  <-- WRONG" };
        report.push(format!("  {label:<28} found {found}, expected {expected}{fail}"));
    }
    println!("{name}\n{}", report.join("\n"));
}

fn main() {
    check("unit", unit::Store::<usize, _>::new(Global::new()).unwrap());
    check("split", split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap());
}
