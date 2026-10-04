"""Turns Criterion's bencher output into the README results and charts.

Input: `results/<background links>-<backend>.txt`, the output of
`cargo bench --bench bench -- --output-format bencher` for one number of
background links and one backend (`neo4j` or `doublets`).

Output, for every number of background links:
- a Markdown table in `results.md`, also written into `../README.md` between
  the `<!-- results:start -->` and `<!-- results:end -->` markers;
- `../Docs/bench_rust_<background links>.png` (linear scale) and
  `../Docs/bench_rust_log_scale_<background links>.png` (log scale).

`BENCHMARK_LINKS` must be set to the value used by the benchmarks.
"""

import logging
import os
import re
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

# Enable detailed tracing. Set to False to disable verbose output.
DEBUG = True
logging.basicConfig(level=logging.INFO if DEBUG else logging.WARNING, format="%(message)s")

RESULTS_DIR = Path("results")
DOCS_DIR = Path("../Docs")
README = Path("../README.md")
START_MARKER = "<!-- results:start -->"
END_MARKER = "<!-- results:end -->"

OPERATIONS = [
    "Create", "Update", "Delete",
    "Each All", "Each Identity", "Each Concrete", "Each Outgoing", "Each Incoming",
]

# Benchmark id suffix, column name and chart color of every implementation.
IMPLEMENTATIONS = [
    ("Doublets_United_Volatile", "Doublets United Volatile", "salmon"),
    ("Doublets_United_NonVolatile", "Doublets United NonVolatile", "red"),
    ("Doublets_Split_Volatile", "Doublets Split Volatile", "lightgreen"),
    ("Doublets_Split_NonVolatile", "Doublets Split NonVolatile", "green"),
    ("Neo4j_NonTransaction", "Neo4j NonTransaction", "lightblue"),
    ("Neo4j_Transaction", "Neo4j Transaction", "blue"),
]

# For example: `test Create/Neo4j_Transaction ... bench:    44055505 ns/iter (+/- 5345991)`
BENCH_LINE = re.compile(r"test\s+(\w+)/(\w+)\s+\.\.\.\s+bench:\s+(\d+)\s+ns/iter")


def read_results():
    """Returns {background links: {(operation, implementation): ns}}."""
    results = {}
    for path in sorted(RESULTS_DIR.glob("*.txt")):
        background = int(path.stem.split("-")[0])
        times = results.setdefault(background, {})
        for group, implementation, ns in BENCH_LINE.findall(path.read_text()):
            operation = group.replace("_", " ")
            times[(operation, implementation)] = int(ns)
            logging.info("%s links, %s, %s: %s ns", background, operation, implementation, ns)
    return dict(sorted(results.items()))


def series(times, implementation):
    """Times of `implementation` in the order of OPERATIONS (0 when missing)."""
    return [times.get((operation, implementation), 0) for operation in OPERATIONS]


def markdown_table(times):
    header = "| Operation | " + " | ".join(name for _, name, _ in IMPLEMENTATIONS) + " |"
    separator = "|---|" + "---:|" * len(IMPLEMENTATIONS)
    rows = [header, separator]
    for operation in OPERATIONS:
        cells = [times.get((operation, implementation)) for implementation, _, _ in IMPLEMENTATIONS]
        rows.append(
            f"| {operation} | " + " | ".join(f"{cell:,}" if cell else "N/A" for cell in cells) + " |"
        )
    return "\n".join(rows)


def chart(times, title, path, log_scale):
    y, width = np.arange(len(OPERATIONS)), 0.13
    fig, ax = plt.subplots(figsize=(12, 8))
    values = [series(times, implementation) for implementation, _, _ in IMPLEMENTATIONS]
    if not log_scale:
        # Bars of 0.5% of the longest bar stay visible (about 4 pixels).
        min_visible = max(max(v) for v in values) * 0.005
        values = [[max(t, min_visible) if t else 0 for t in v] for v in values]
    for index, ((_, name, color), times_ns) in enumerate(zip(IMPLEMENTATIONS, values)):
        offset = (index - (len(IMPLEMENTATIONS) - 1) / 2) * width
        ax.barh(y + offset, times_ns, width, label=name, color=color)
    ax.set_xlabel("Time of one iteration (ns)" + (", log scale" if log_scale else ""))
    if log_scale:
        ax.set_xscale("log")
    ax.set_title(title)
    ax.set_yticks(y)
    ax.set_yticklabels(OPERATIONS)
    ax.invert_yaxis()
    ax.legend(loc="upper left", bbox_to_anchor=(1.01, 1))
    fig.tight_layout()
    fig.savefig(path)
    plt.close(fig)
    logging.info("%s saved", path)


def main():
    links = int(os.environ["BENCHMARK_LINKS"])
    results = read_results()
    if not results:
        raise SystemExit(f"no results found in {RESULTS_DIR}/")
    DOCS_DIR.mkdir(exist_ok=True)

    sections = []
    for background, times in results.items():
        title = f"{background:,} background links, {links:,} links per iteration"
        linear = DOCS_DIR / f"bench_rust_{background}.png"
        log = DOCS_DIR / f"bench_rust_log_scale_{background}.png"
        chart(times, title, linear, log_scale=False)
        chart(times, title, log, log_scale=True)
        sections.append(
            f"### {title}\n\n"
            f"Median time of one iteration in nanoseconds. In the linear chart, bars "
            f"shorter than 0.5% of the longest bar are drawn 0.5% long to stay visible.\n\n"
            f"{markdown_table(times)}\n\n"
            f"![{title}, linear scale](Docs/{linear.name})\n"
            f"![{title}, log scale](Docs/{log.name})"
        )
    results_md = "\n\n".join(sections)
    Path("results.md").write_text(results_md + "\n")
    print(results_md)

    readme = README.read_text()
    start, end = readme.index(START_MARKER) + len(START_MARKER), readme.index(END_MARKER)
    README.write_text(readme[:start] + "\n" + results_md + "\n" + readme[end:])
    logging.info("%s updated", README)


if __name__ == "__main__":
    main()
