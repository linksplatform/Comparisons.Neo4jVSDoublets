//! Measures per-operation latency of basic link operations in Neo4j through the
//! `neo4rs` Bolt driver (persistent pooled connection).
//! Run with: NEO4J_URI=bolt://localhost:7687 cargo run --release
use std::time::{Duration, Instant};

use neo4rs::{query, Graph};

const N: i64 = 300;

fn report(name: &str, total: Duration, n: i64) {
    println!("{name:<40} {:>10.1} µs/op", total.as_secs_f64() * 1e6 / n as f64);
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".into());
    let graph = Graph::new(uri, "neo4j", "password").await?;
    graph.run(query("CREATE CONSTRAINT link_id IF NOT EXISTS FOR (l:Link) REQUIRE l.id IS UNIQUE")).await?;
    graph.run(query("CREATE INDEX link_source IF NOT EXISTS FOR (l:Link) ON (l.source)")).await?;
    graph.run(query("CREATE INDEX link_target IF NOT EXISTS FOR (l:Link) ON (l.target)")).await?;
    graph.run(query("MATCH (l:Link) DETACH DELETE l")).await?;

    // warm up
    for id in 1..=N {
        graph.run(query("CREATE (:Link {id: $id, source: $id, target: $id})").param("id", id)).await?;
    }
    graph.run(query("MATCH (l:Link) DETACH DELETE l")).await?;

    let start = Instant::now();
    for id in 1..=N {
        graph.run(query("CREATE (:Link {id: $id, source: $id, target: $id})").param("id", id)).await?;
    }
    report("auto-commit create_point (run)", start.elapsed(), N);

    let start = Instant::now();
    for id in 1..=N {
        let mut rows = graph.execute(query("MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target").param("id", id)).await?;
        while let Some(_row) = rows.next().await? {}
    }
    report("auto-commit read by id (execute)", start.elapsed(), N);

    let start = Instant::now();
    for id in 1..=N {
        let mut rows = graph.execute(query("MATCH (l:Link {id: $id}) WITH l, l.source AS s, l.target AS t SET l.source = 0, l.target = 0 RETURN s, t").param("id", id)).await?;
        while let Some(_row) = rows.next().await? {}
    }
    report("auto-commit update (execute)", start.elapsed(), N);

    let start = Instant::now();
    let mut txn = graph.start_txn().await?;
    for id in 1..=N {
        let mut rows = txn.execute(query("MATCH (l:Link {id: $id}) WITH l, l.source AS s, l.target AS t SET l.source = $id, l.target = $id RETURN s, t").param("id", id)).await?;
        while let Some(_row) = rows.next(txn.handle()).await? {}
    }
    txn.commit().await?;
    report("explicit txn update (execute) + commit", start.elapsed(), N);

    let start = Instant::now();
    let mut txn = graph.start_txn().await?;
    for id in 1..=N {
        let mut rows = txn.execute(query("MATCH (l:Link {id: $id}) RETURN l.id, l.source, l.target").param("id", id)).await?;
        while let Some(_row) = rows.next(txn.handle()).await? {}
    }
    txn.commit().await?;
    report("explicit txn read by id", start.elapsed(), N);

    let ids: Vec<i64> = (1..=N).collect();
    let start = Instant::now();
    graph.run(query("UNWIND $ids AS id MATCH (l:Link {id: id}) SET l.source = 0, l.target = 0").param("ids", ids.clone())).await?;
    report("batch UNWIND update", start.elapsed(), N);

    let start = Instant::now();
    for id in 1..=N {
        let mut rows = graph.execute(query("MATCH (l:Link {id: $id}) WITH l, l.source AS s, l.target AS t DELETE l RETURN s, t").param("id", id)).await?;
        while let Some(_row) = rows.next().await? {}
    }
    report("auto-commit delete (execute)", start.elapsed(), N);

    let start = Instant::now();
    graph.run(query("UNWIND $ids AS id CREATE (:Link {id: id, source: id, target: id})").param("ids", ids.clone())).await?;
    report("batch UNWIND create", start.elapsed(), N);
    graph.run(query("MATCH (l:Link) DETACH DELETE l")).await?;
    Ok(())
}
