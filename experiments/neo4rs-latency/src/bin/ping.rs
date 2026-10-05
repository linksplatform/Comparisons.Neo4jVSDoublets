//! Measures the round-trip latency of a trivial `RETURN 1` query through `neo4rs`.
use std::time::Instant;

use neo4rs::{query, Graph};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".into());
    let graph = Graph::new(uri, "neo4j", "password").await?;
    for round in 0..3 {
        let n = 1000;
        let start = Instant::now();
        for _ in 0..n {
            graph.run(query("RETURN 1")).await?;
        }
        println!("round {round}: run RETURN 1 {:.1} µs/op", start.elapsed().as_secs_f64() * 1e6 / n as f64);
        let mut txn = graph.start_txn().await?;
        let start = Instant::now();
        for _ in 0..n {
            txn.run(query("RETURN 1")).await?;
        }
        println!("round {round}: txn RETURN 1 {:.1} µs/op", start.elapsed().as_secs_f64() * 1e6 / n as f64);
        txn.commit().await?;
    }
    Ok(())
}
