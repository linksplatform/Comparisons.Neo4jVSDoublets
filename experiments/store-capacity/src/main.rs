//! Finds how many point links each doublets 0.5.0 store can hold.
//!
//! Usage: `store-capacity <store> <count>`, where `<store>` is `unit-ram`, `unit-file`,
//! `split-ram` or `split-file`. Creates `<count>` point links and prints the progress,
//! or panics inside doublets when the store cannot grow.
use std::fs::File;

use doublets::{
    mem::{FileMapped, Global},
    split, unit, Doublets,
};

fn mapped<T: Default>(path: &str) -> FileMapped<T> {
    let file = File::options().create(true).truncate(true).read(true).write(true).open(path).unwrap();
    FileMapped::new(file).unwrap()
}

fn fill(store: &mut impl Doublets<usize>, count: usize) -> Result<(), doublets::Error<usize>> {
    for created in 1..=count {
        store.create_point()?;
        if created.is_power_of_two() || created == count {
            println!("{created} links created");
        }
    }
    Ok(())
}

fn main() -> Result<(), doublets::Error<usize>> {
    let mut args = std::env::args().skip(1);
    let store = args.next().expect("store kind");
    let count: usize = args.next().map_or(10_000_000, |n| n.parse().unwrap());
    match store.as_str() {
        "unit-ram" => fill(&mut unit::Store::<usize, _>::new(Global::new())?, count),
        "unit-file" => fill(&mut unit::Store::<usize, _>::new(mapped("unit.links"))?, count),
        "split-ram" => fill(&mut split::Store::<usize, _, _>::new(Global::new(), Global::new())?, count),
        "split-file" => fill(
            &mut split::Store::<usize, _, _>::new(mapped("data.links"), mapped("index.links"))?,
            count,
        ),
        _ => panic!("unknown store `{store}`"),
    }
}
