//! Prints how many network round trips `neo4rs` needs per query in each usage mode.
//! Requires `experiments/bolt-round-trips/proxy.py` listening on localhost:7688.
use neo4rs::{query, Graph};

fn flights() -> u64 {
    let pid = std::env::var("PROXY_PID").expect("set PROXY_PID to the proxy.py process id");
    std::process::Command::new("kill").args(["-USR1", &pid]).status().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(200));
    let log = std::fs::read_to_string("../bolt-round-trips/proxy.log").unwrap();
    log.lines().last().unwrap().trim_start_matches("flights=").parse().unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph = Graph::new("bolt://127.0.0.1:7688", "neo4j", "password").await?;
    graph.run(query("RETURN 1")).await?;
    let n = 100;

    let before = flights();
    for _ in 0..n {
        graph.run(query("RETURN 1")).await?;
    }
    println!("Graph::run           {:.2} round trips/query", (flights() - before) as f64 / n as f64);

    let before = flights();
    for _ in 0..n {
        let mut rows = graph.execute(query("RETURN 1 AS x")).await?;
        while rows.next().await?.is_some() {}
    }
    println!("Graph::execute       {:.2} round trips/query", (flights() - before) as f64 / n as f64);

    let before = flights();
    let mut txn = graph.start_txn().await?;
    for _ in 0..n {
        txn.run(query("RETURN 1")).await?;
    }
    txn.commit().await?;
    println!("Txn::run (+begin/commit) {:.2} round trips/query", (flights() - before) as f64 / n as f64);

    let before = flights();
    let mut txn = graph.start_txn().await?;
    for _ in 0..n {
        let mut rows = txn.execute(query("RETURN 1 AS x")).await?;
        while rows.next(txn.handle()).await?.is_some() {}
    }
    txn.commit().await?;
    println!("Txn::execute (+begin/commit) {:.2} round trips/query", (flights() - before) as f64 / n as f64);
    Ok(())
}
