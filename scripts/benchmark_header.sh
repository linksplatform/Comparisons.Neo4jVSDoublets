#!/usr/bin/env bash
# Prints the `# key: value` lines that describe a benchmark run, for
# `scripts/benchmark_report.py`. Usage: benchmark_header.sh neo4j|doublets
set -euo pipefail

echo "# links: ${BENCHMARK_LINKS:-100}"
echo "# background links: ${BENCHMARK_BACKGROUND_LINKS:-1000}"
cpu=$(lscpu 2>/dev/null | sed -n 's/^Model name: *//p' | head -n 1)
echo "# cpu: ${cpu:-unknown}"
if [ "${1:-}" = neo4j ]; then
  # The HTTP discovery endpoint returns {"neo4j_version": "2026.09.0", ...}.
  version=$(curl -fsS "${NEO4J_HTTP:-http://localhost:7474}" | sed -n 's/.*"neo4j_version":"\([^"]*\)".*/\1/p')
  edition=$(curl -fsS "${NEO4J_HTTP:-http://localhost:7474}" | sed -n 's/.*"neo4j_edition":"\([^"]*\)".*/\1/p')
  echo "# neo4j: ${version:-unknown version} ${edition^}"
fi
echo "# date: $(date -u +%Y-%m-%d)"
if [ -n "${GITHUB_RUN_ID:-}" ]; then
  echo "# run: ${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"
fi
