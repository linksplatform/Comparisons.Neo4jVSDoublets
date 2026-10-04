# Experiments

Checks made while preparing the benchmarks of
[issue #15](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/issues/15).
They are not part of the CI. Outputs are in [`logs`](logs).

| Directory | What it checks |
|-----------|----------------|
| [`bolt-round-trips`](bolt-round-trips) | TCP proxy (`localhost:7688` → `localhost:7687`) that counts Bolt round trips |
| [`neo4rs-latency`](neo4rs-latency) | latency of link operations through neo4rs; `round_trips` binary: round trips per query |
| [`neo4j-crate-latency`](neo4j-crate-latency) | the same for the `neo4j` crate |
| [`split-store-bug`](split-store-bug) | wrong query results of the doublets 0.5.0 Split store after `update` |

Round trips per query, measured through the proxy
([`logs/neo4rs-round-trips.log`](logs/neo4rs-round-trips.log),
[`logs/neo4j-crate-round-trips.log`](logs/neo4j-crate-round-trips.log)):

| Driver | Outside of a transaction | Inside an explicit transaction |
|--------|-------------------------:|-------------------------------:|
| neo4rs 0.8 | 3 | 2 |
| neo4j 0.2 | 2 | 1 |

Counting round trips (Neo4j running on `localhost:7687` with password `password`):

```bash
cd experiments/bolt-round-trips
python3 proxy.py > proxy.log &
cd ../neo4rs-latency
PROXY_PID=$(pgrep -f proxy.py) cargo run --release --bin round_trips
```
