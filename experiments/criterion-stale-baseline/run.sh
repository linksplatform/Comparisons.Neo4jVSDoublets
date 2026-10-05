#!/usr/bin/env bash
# Reproduces the CI failure of runs 37244515815 and 37245095420: rust-cache
# restored target/criterion/<id>/base without the JSON files, Criterion tried
# to compare with it and printed an error in the middle of the bencher line.
#
# Run from the repository root: experiments/criterion-stale-baseline/run.sh
set -euo pipefail
cd rust
export BENCHMARK_BACKEND=doublets BENCHMARK_LINKS=100 BENCHMARK_BACKGROUND_LINKS=1000
filter='Create/Doublets_United_Volatile'
stale=$(mktemp -d)

echo "== a base directory without files (as restored by the cache)"
mkdir -p "$stale/Create/Doublets_United_Volatile/base"
CRITERION_HOME="$stale" cargo bench -q --bench bench -- --output-format bencher --noplot "$filter" 2>&1 | grep -A1 '^test'

echo "== a fresh directory (as the CI uses now)"
fresh=$(mktemp -d)
CRITERION_HOME="$fresh" cargo bench -q --bench bench -- --output-format bencher --noplot "$filter" 2>&1 | grep -A1 '^test'
rm -rf "$stale" "$fresh"
