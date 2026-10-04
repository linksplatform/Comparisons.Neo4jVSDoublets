//! Measures the round-trip latency of a trivial `RETURN 1` query through the `neo4j` crate
//! (synchronous driver that mirrors the official drivers' API).
use std::sync::Arc;
use std::time::Instant;

use neo4j::address::Address;
use neo4j::driver::auth::AuthToken;
use neo4j::driver::{ConnectionConfig, Driver, DriverConfig};
use neo4j::session::SessionConfig;
use neo4j::transaction::Transaction;

fn main() {
    let host = std::env::var("NEO4J_HOST").unwrap_or_else(|_| "localhost".into());
    let driver = Driver::new(
        ConnectionConfig::new(Address::from((host.as_str(), 7687))),
        DriverConfig::new().with_auth(Arc::new(AuthToken::new_basic_auth("neo4j", "password"))),
    );
    let database = Arc::new(String::from("neo4j"));
    let mut session = driver.session(SessionConfig::new().with_database(Arc::clone(&database)));
    for round in 0..3 {
        let n = 1000;
        let start = Instant::now();
        for _ in 0..n {
            session.auto_commit("RETURN 1").run().unwrap();
        }
        println!("round {round}: auto_commit RETURN 1 {:.1} µs/op", start.elapsed().as_secs_f64() * 1e6 / n as f64);
        let start = Instant::now();
        for _ in 0..n {
            driver.execute_query("RETURN 1").with_database(Arc::clone(&database)).run().unwrap();
        }
        println!("round {round}: execute_query RETURN 1 {:.1} µs/op", start.elapsed().as_secs_f64() * 1e6 / n as f64);
        let start = Instant::now();
        session
            .transaction()
            .run(|tx: Transaction| {
                for _ in 0..n {
                    tx.query("RETURN 1").run()?.consume()?;
                }
                tx.commit()
            })
            .unwrap();
        println!("round {round}: txn RETURN 1 {:.1} µs/op", start.elapsed().as_secs_f64() * 1e6 / n as f64);
    }
}
