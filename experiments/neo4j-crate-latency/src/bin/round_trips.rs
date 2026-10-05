//! Prints how many network round trips the `neo4j` crate needs per query in each usage mode.
//! Requires `experiments/bolt-round-trips/proxy.py` listening on localhost:7688.
use std::sync::Arc;

use neo4j::address::Address;
use neo4j::driver::auth::AuthToken;
use neo4j::driver::{ConnectionConfig, Driver, DriverConfig};
use neo4j::session::SessionConfig;
use neo4j::transaction::Transaction;

fn flights() -> u64 {
    let pid = std::env::var("PROXY_PID").expect("set PROXY_PID to the proxy.py process id");
    std::process::Command::new("kill").args(["-USR1", &pid]).status().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let log = std::fs::read_to_string("../bolt-round-trips/proxy.log").unwrap();
    log.lines().last().unwrap().trim_start_matches("flights=").parse().unwrap()
}

fn main() {
    let driver = Driver::new(
        ConnectionConfig::new(Address::from(("127.0.0.1", 7688))),
        DriverConfig::new().with_auth(Arc::new(AuthToken::new_basic_auth("neo4j", "password"))),
    );
    let database = Arc::new(String::from("neo4j"));
    let mut session = driver.session(SessionConfig::new().with_database(Arc::clone(&database)));
    session.auto_commit("RETURN 1").run().unwrap();
    let n = 100;

    let before = flights();
    for _ in 0..n {
        session.auto_commit("RETURN 1").run().unwrap();
    }
    println!("Session::auto_commit  {:.2} round trips/query", (flights() - before) as f64 / n as f64);

    let before = flights();
    for _ in 0..n {
        driver.execute_query("RETURN 1").with_database(Arc::clone(&database)).run().unwrap();
    }
    println!("Driver::execute_query {:.2} round trips/query", (flights() - before) as f64 / n as f64);

    let before = flights();
    session
        .transaction()
        .run(|tx: Transaction| {
            for _ in 0..n {
                tx.query("RETURN 1").run()?.consume()?;
            }
            tx.commit()
        })
        .unwrap();
    println!("Transaction::query (+begin/commit) {:.2} round trips/query", (flights() - before) as f64 / n as f64);
}
