# Case Study: Issue #5 - Benchmark is Broken

## Executive Summary

The benchmark workflow reports false success even when the benchmark fails with a panic. This case study analyzes the root causes, timeline of events, and proposes solutions.

**Primary Issue:** GitHub Actions workflow incorrectly reports success despite `cargo bench` failing with a panic.

**Root Causes Identified:**
1. **Pipeline Exit Code Masking:** Using `| tee out.txt` in the benchmark command masks the exit code
2. **Benchmark Logic Bug:** The delete benchmark attempts to delete links that don't exist due to shared state between benchmarks

## Timeline of Events

| Timestamp (UTC) | Event |
|-----------------|-------|
| 2025-12-22 19:49:32 | PR #4 merged: "Fix benchmark failure: Transaction implementation was a broken stub" |
| 2025-12-22 19:49:39 | CI run #20442351221 starts |
| 2025-12-22 19:55:19 | `cargo bench` begins execution |
| 2025-12-22 20:18:16 | Create/Neo4j_NonTransaction completes |
| 2025-12-22 20:39:35 | Create/Neo4j_Transaction completes |
| 2025-12-22 20:40:26 | All Doublets Create benchmarks complete |
| 2025-12-22 21:05:20 | Delete/Neo4j_NonTransaction completes (1988ms/iter) |
| 2025-12-22 21:05:47 | **PANIC:** `NotExists(4000)` at `delete.rs:23` |
| 2025-12-22 21:05:47 | `error: bench failed` message displayed |
| 2025-12-22 21:05:56 | "Prepare benchmark results" step runs (despite failure) |
| 2025-12-22 21:05:56 | Commit 5134f36: "Update benchmark results" pushed to main |
| 2025-12-22 21:05:57 | CI run completes with **SUCCESS** status |

## Root Cause Analysis

### Root Cause #1: Pipeline Exit Code Masking

**Location:** `.github/workflows/rust.yml:60`

```yaml
run: cargo bench --bench bench -- --output-format bencher | tee out.txt
```

**Problem:** When a command is piped to `tee`, the exit code of the entire pipeline is determined by the last command (`tee`), not the first command (`cargo bench`). Since `tee` successfully writes output to the file, it returns exit code 0 even though `cargo bench` returned exit code 101.

**Evidence:**
- CI logs show `error: bench failed` message
- The step "Run benchmark" shows `conclusion: success` in GitHub Actions API
- Subsequent steps (Prepare benchmark results) execute as if nothing failed

**Solution:** Use `set -o pipefail` to ensure the pipeline returns the exit code of the first failing command:

```yaml
run: |
  set -o pipefail
  cargo bench --bench bench -- --output-format bencher | tee out.txt
```

Or use bash-specific syntax:
```yaml
shell: bash
run: |
  set -o pipefail
  cargo bench --bench bench -- --output-format bencher | tee out.txt
```

### Root Cause #2: Benchmark Logic Bug (NotExists Error)

**Location:** `rust/benches/benchmarks/delete.rs:23-28`

```rust
for id in (BACKGROUND_LINKS..=BACKGROUND_LINKS + 1_000).rev() {
    let _ = elapsed! {fork.delete(id)?};
}
```

**Problem:** The delete benchmark tries to delete links with IDs from 3000 to 4001, but the Transaction benchmark shares state with the Client benchmark. Here's the sequence:

1. `Delete/Neo4j_NonTransaction` runs successfully, creating links with IDs 1-4001 and deleting them
2. `Delete/Neo4j_Transaction` starts - but it uses a shared `Client` that was initialized during `create` benchmarks
3. The client's `next_id` counter has been modified by previous benchmarks
4. When trying to delete ID 4000, it doesn't exist (was never created or was deleted)

**Evidence from logs:**
```
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: NotExists(4000)', benches/benchmarks/delete.rs:23:9
```

**Solution Options:**
1. Ensure Transaction cleanup properly resets state between benchmarks
2. Use proper test isolation with separate database instances
3. Handle the `NotExists` error gracefully in benchmarks

### Root Cause #3: Flawed Transaction Implementation

**Location:** `rust/src/benched.rs:94-108`

```rust
impl<'a, T: LinkType> Benched for Exclusive<Transaction<'a, T>> {
    // ...
    fn fork(&mut self) -> Fork<Self> {
        Fork(self)  // Does not call create_table()!
    }

    unsafe fn unfork(&mut self) {
        // Transaction cleanup handled by client
    }
}
```

**Problem:** Unlike the `Client` implementation which calls `create_table()` in `fork()`, the `Transaction` implementation does not set up a clean state. This means:
- Data from previous benchmarks persists
- The `next_id` counter is not reset
- Links created by previous tests affect the Transaction benchmark

**Contrast with Client implementation:**
```rust
impl<T: LinkType> Benched for Exclusive<Client<T>> {
    fn fork(&mut self) -> Fork<Self> {
        let _ = self.create_table();  // Proper setup
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.drop_table();  // Proper cleanup
    }
}
```

## Proposed Solutions

### Solution 1: Fix Pipeline Exit Code (Critical)

**File:** `.github/workflows/rust.yml`

Change:
```yaml
- name: Run benchmark
  working-directory: rust
  env:
    NEO4J_URI: bolt://localhost:7687
    NEO4J_USER: neo4j
    NEO4J_PASSWORD: password
  run: cargo bench --bench bench -- --output-format bencher | tee out.txt
```

To:
```yaml
- name: Run benchmark
  working-directory: rust
  env:
    NEO4J_URI: bolt://localhost:7687
    NEO4J_USER: neo4j
    NEO4J_PASSWORD: password
  run: |
    set -o pipefail
    cargo bench --bench bench -- --output-format bencher | tee out.txt
```

### Solution 2: Fix Transaction Benchmark Isolation

**File:** `rust/src/benched.rs`

Update the Transaction `fork()` method to properly clean state:

```rust
impl<'a, T: LinkType> Benched for Exclusive<Transaction<'a, T>> {
    // ...
    fn fork(&mut self) -> Fork<Self> {
        let _ = self.drop_table();  // Clean previous state
        Fork(self)
    }

    unsafe fn unfork(&mut self) {
        let _ = self.drop_table();  // Clean after benchmark
    }
}
```

### Solution 3: Make Benchmarks Independent (Long-term)

Consider restructuring the benchmarks to:
1. Each benchmark creates its own isolated database/table
2. Use unique prefixes or namespaces for each benchmark run
3. Add proper cleanup verification before each benchmark

## Impact Assessment

| Impact Area | Severity | Description |
|-------------|----------|-------------|
| CI Reliability | High | False success leads to pushing broken code |
| Data Integrity | Medium | Incorrect benchmark results published to README |
| Developer Trust | High | Can't rely on CI to catch benchmark failures |

## Files Changed in This Incident

1. `Docs/bench_rust.png` - Updated with partial results
2. `Docs/bench_rust_log_scale.png` - Updated with partial results
3. `README.md` - Results table updated with incomplete data showing N/A for failed benchmarks

## References

- [GitHub Actions: Handling Step Errors](https://www.kenmuse.com/blog/how-to-handle-step-and-job-errors-in-github-actions/)
- [Bash pipefail option](https://www.gnu.org/software/bash/manual/bash.html#The-Set-Builtin)
- [cargo bench documentation](https://doc.rust-lang.org/cargo/commands/cargo-bench.html)
- [CI Run #20442351221](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/actions/runs/20442351221/job/58738054640)
- [Commit 5134f36 (pushed with broken benchmarks)](https://github.com/linksplatform/Comparisons.Neo4jVSDoublets/commit/5134f36730caa5a6b571692e4b67b1e3cb523e3e)

## Appendix: Full Error Log Extract

```
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 1450.3s, or reduce sample count to 10.
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: NotExists(4000)', benches/benchmarks/delete.rs:23:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test Delete/Neo4j_Transaction ...
error: bench failed
```
