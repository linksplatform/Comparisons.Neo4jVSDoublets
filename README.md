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
| Create        | 81055 (42346.8x faster)  | 83846 (40937.2x faster)     | 72568 (47299.4x faster) | 71612 (47930.8x faster)    | 3459139258           | 3432423917        |
| Update        | 1308 (33217.3x faster)   | 1344 (32327.6x faster)      | 674 (64463.3x faster)   | 711 (61108.7x faster)      | 43708416             | 43448259          |
| Delete        | 130634 (16988.3x faster) | 130571 (16996.5x faster)    | 132360 (16766.8x faster) | 134435 (16508.0x faster)   | 2235068799           | 2219251214        |
| Each All      | 63 (16560.2x faster)     | 71 (14694.3x faster)        | 81 (12880.2x faster)    | 80 (13041.2x faster)       | 1062528              | 1043293           |
| Each Identity | 240 (42019.7x faster)    | 234 (43097.1x faster)       | 245 (41162.2x faster)   | 255 (39548.0x faster)      | 10097022             | 10084733          |
| Each Concrete | 346 (29146.2x faster)    | 348 (28978.7x faster)       | 378 (26678.8x faster)   | 370 (27255.6x faster)      | 10138701             | 10084574          |
| Each Outgoing | 437 (22980.0x faster)    | 436 (23032.7x faster)       | 339 (29623.2x faster)   | 330 (30431.1x faster)      | 10042277             | 10066860          |
| Each Incoming | 444 (22453.1x faster)    | 440 (22657.2x faster)       | 359 (27769.3x faster)   | 357 (27924.9x faster)      | 10094595             | 9969185           |

## Conclusion

The benchmark results will be automatically updated once the GitHub Actions workflow runs. Results are expected to show that Doublets significantly outperforms Neo4j in both write and read operations due to its specialized data structure for storing doublet-links.

To get fresh numbers, please fork the repository and rerun benchmark in GitHub Actions.
