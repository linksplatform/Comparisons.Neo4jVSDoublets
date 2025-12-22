# Comparisons.Neo4jVSDoublets

The comparison between Neo4j and LinksPlatform's Doublets (links) on basic database operations with links (create, read, delete, update).
All benchmarks ran with 3000 links in background to increase size of indexes and 1000 are actively created/updated/deleted.

In this particular benchmark we decided not to increase the number of links as Neo4j will not be able to handle it at all in timeframe what GitHub Actions limit allows to use for free. Remember that to get accurate result we ran this benchmark multiple times.

## Task

Both databases used to store and retrieve doublet-links representation. To support storage, and all basic CRUD operations that provide Turing completeness for links as in [the links theory](https://habr.com/ru/articles/895896).

## Operations
- **Create** – insert point link (link with id = source = target)
- **Update** – basic link update operation
- **Delete** – basic link delete operation
- **Each All** – take all links matching `[*, *, *]` constraint
- **Each Incoming** – take all links matching `[*, *, target]` constraint
- **Each Outgoing** – take all links matching `[*, source, *]` constraint
- **Each Concrete** – take all links matching `[*, source, target]` constraint
- **Each Identity** – take all links matching `[id, *, *]` constraint

## Results
The results below represent the amount of time (ns) the operation takes per iteration.
- First picture shows time in a pixel scale (for doublets just minimum value is shown, otherwise it will be not present on the graph).
- Second picture shows time in a logarithmic scale (to see difference clearly, because it is around 2-3 orders of magnitude).

### Rust
![Image of Rust benchmark (pixel scale)](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/blob/main/Docs/bench_rust.png?raw=true)
![Image of Rust benchmark (log scale)](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/blob/main/Docs/bench_rust_log_scale.png?raw=true)

### Raw benchmark results (all numbers are in nanoseconds)

| Operation     | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction |
|---------------|--------------------------|-----------------------------|-------------------------|----------------------------|----------------------|-------------------|
| Create        | 97789 (0.3x faster)      | 100438 (0.3x faster)        | 86465 (0.4x faster)     | 83558 (0.4x faster)        | 3107850791           | 31740             |
| Update        | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |
| Delete        | N/A                      | N/A                         | N/A                     | N/A                        | 1868497988           | N/A               |
| Each All      | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |
| Each Identity | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |
| Each Concrete | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |
| Each Outgoing | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |
| Each Incoming | N/A                      | N/A                         | N/A                     | N/A                        | N/A                  | N/A               |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
