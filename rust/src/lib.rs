#![feature(allocator_api, generic_associated_types)]

#[macro_export]
macro_rules! bench {
    {|$fork:ident| as $B:ident { $($body:tt)* }} => {
        (move |bencher: &mut criterion::Bencher, benched: &mut _| {
            bencher.iter_custom(|iters| {
                let mut __bench_duration = Duration::ZERO;
                macro_rules! elapsed {
                    {$expr:expr} => {{
                        let __instant = Instant::now();
                        let __ret = {$expr};
                        __bench_duration += __instant.elapsed();
                        __ret
                    }};
                }
                crate::tri! {
                    use linksneo4j::BACKGROUND_LINKS;
                    for _iter in 0..iters {
                        let mut $fork: Fork<$B> = Benched::fork(&mut *benched);
                        for _ in 0..BACKGROUND_LINKS {
                            let _ = $fork.create_point()?;
                        }
                        $($body)*
                    }
                }
                __bench_duration
            });
        })
    }
}

use std::{error, fs::File, io, result};

pub use benched::Benched;
pub use client::Client;
use doublets::{data::LinkType, mem::FileMapped};
pub use exclusive::Exclusive;
pub use fork::Fork;
pub use transaction::Transaction;

mod benched;
mod client;
mod exclusive;
mod fork;
mod transaction;

pub type Result<T, E = Box<dyn error::Error + Sync + Send>> = result::Result<T, E>;

/// Number of background links to create before each benchmark iteration.
/// This simulates a database with existing data.
pub const BACKGROUND_LINKS: usize = 10;

/// Number of links to create/delete/update in each benchmark operation.
/// Can be configured via BENCHMARK_LINK_COUNT environment variable.
/// Defaults to 10 for faster iteration in pull requests, but should be set to 1000 for main branch benchmarks.
pub fn link_count() -> usize {
    std::env::var("BENCHMARK_LINK_COUNT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// Lazy static to cache the link count value
pub use once_cell::sync::Lazy;
pub static LINK_COUNT: Lazy<usize> = Lazy::new(link_count);

/// Connect to Neo4j database
pub fn connect<T: LinkType>() -> Result<Client<T>> {
    // Default Neo4j connection parameters
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
    Client::new(&uri, &user, &password)
}

pub fn map_file<T: Default>(filename: &str) -> io::Result<FileMapped<T>> {
    let file = File::options()
        .create(true)
        .write(true)
        .read(true)
        .open(filename)?;
    FileMapped::new(file)
}

pub trait Sql {
    fn create_table(&mut self) -> Result<()>;
    fn drop_table(&mut self) -> Result<()>;
}
