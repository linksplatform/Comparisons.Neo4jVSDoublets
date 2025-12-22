import re
import logging
import matplotlib.pyplot as plt
import numpy as np

# Enable detailed tracing. Set to False to disable verbose output.
DEBUG = True
logging.basicConfig(level=logging.INFO if DEBUG else logging.WARNING,
                    format="%(message)s")

# Read the complete file
data = open("out.txt").read()
if DEBUG:
    logging.info("Loaded out.txt, length: %d characters", len(data))

# Use two regex patterns to extract benchmarks.
# The first captures Neo4j tests and the second captures Doublets tests.
patterns = [
    r"test\s+(\w+)/(Neo4j)_(\w+)\s+\.\.\.\s+bench:\s+(\d+)\s+ns/iter\s+\(\+/-\s+\d+\)",
    r"test\s+(\w+)/(Doublets)_(\w+)_(\w+)\s+\.\.\.\s+bench:\s+(\d+)\s+ns/iter\s+\(\+/-\s+\d+\)"
]

# Instead of using lists, we use dictionaries mapping operation names to values.
Neo4j_Transaction = {}
Neo4j_NonTransaction = {}
Doublets_United_Volatile = {}
Doublets_United_NonVolatile = {}
Doublets_Split_Volatile = {}
Doublets_Split_NonVolatile = {}

# Process each regex pattern
for pattern in patterns:
    matches = re.findall(pattern, data)
    if DEBUG:
        logging.info("Pattern %s matched %d entries", pattern, len(matches))
    for match in matches:
        # Normalise name
        op = match[0].replace("_", " ")  # Create, Each All, …
        if match[1] == 'Neo4j':
            # (operation, 'Neo4j', transaction, time)
            transaction = match[2]
            time_val = int(match[3])
            if DEBUG:
                logging.info("Neo4j %s - %s: %d ns", op, transaction, time_val)
            if transaction == "Transaction":
                Neo4j_Transaction[op] = time_val
            else:
                Neo4j_NonTransaction[op] = time_val
        else:
            # (operation, 'Doublets', trees, storage, time)
            trees = match[2]
            storage = match[3]
            time_val = int(match[4])
            if DEBUG:
                logging.info("Doublets %s - %s %s: %d ns", op, trees, storage, time_val)
            if trees == 'United':
                if storage == 'Volatile':
                    Doublets_United_Volatile[op] = time_val
                else:
                    Doublets_United_NonVolatile[op] = time_val
            else:
                if storage == 'Volatile':
                    Doublets_Split_Volatile[op] = time_val
                else:
                    Doublets_Split_NonVolatile[op] = time_val

# Operation order for table and plots
ordered_ops = [
    "Create", "Update", "Delete",
    "Each All", "Each Identity", "Each Concrete", "Each Outgoing", "Each Incoming"
]

if DEBUG:
    logging.info("\nFinal dictionaries (after parsing):")
    logging.info("Neo4j_Transaction: %s", Neo4j_Transaction)
    logging.info("Neo4j_NonTransaction: %s", Neo4j_NonTransaction)
    logging.info("Doublets_United_Volatile: %s", Doublets_United_Volatile)
    logging.info("Doublets_United_NonVolatile: %s", Doublets_United_NonVolatile)
    logging.info("Doublets_Split_Volatile: %s", Doublets_Split_Volatile)
    logging.info("Doublets_Split_NonVolatile: %s", Doublets_Split_NonVolatile)

# Assemble series in the desired order.
def get_series(d): return [d.get(op, 0) for op in ordered_ops]

du_volatile_arr   = get_series(Doublets_United_Volatile)
du_nonvolatile_arr= get_series(Doublets_United_NonVolatile)
ds_volatile_arr   = get_series(Doublets_Split_Volatile)
ds_nonvolatile_arr= get_series(Doublets_Split_NonVolatile)
neo4j_non_arr     = get_series(Neo4j_NonTransaction)
neo4j_trans_arr   = get_series(Neo4j_Transaction)

# ─────────────────────────────────────────────────────────────────────────────
# Markdown Table
# ─────────────────────────────────────────────────────────────────────────────
def print_results_markdown():
    header = (
        "| Operation     | Doublets United Volatile | Doublets United NonVolatile | "
        "Doublets Split Volatile | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction |\n"
        "|---------------|--------------------------|-----------------------------|-------------------------|----------------------------|----------------------|-------------------|"
    )
    lines = [header]

    for i, op in enumerate(ordered_ops):
        neo4j_val1 = neo4j_non_arr[i]   if neo4j_non_arr[i]   else float('inf')
        neo4j_val2 = neo4j_trans_arr[i] if neo4j_trans_arr[i] else float('inf')
        min_neo4j  = min(neo4j_val1, neo4j_val2)

        def annotate(v):
            if v == 0: return "N/A"
            if min_neo4j == float('inf'): return f"{v}"
            return f"{v} ({min_neo4j / v:.1f}x faster)"

        row = (
            f"| {op:<13} | {annotate(du_volatile_arr[i]):<24} | "
            f"{annotate(du_nonvolatile_arr[i]):<27} | "
            f"{annotate(ds_volatile_arr[i]):<23} | "
            f"{annotate(ds_nonvolatile_arr[i]):<26} | "
            f"{neo4j_non_arr[i] or 'N/A':<20} | {neo4j_trans_arr[i] or 'N/A':<17} |"
        )
        lines.append(row)

    table_md = "\n".join(lines)
    print(table_md)

    # Save to file for CI to use
    with open("results.md", "w") as f:
        f.write(table_md)

    if DEBUG: logging.info("\nGenerated Markdown Table:\n%s", table_md)

# ─────────────────────────────────────────────────────────────────────────────
# Plots
# ─────────────────────────────────────────────────────────────────────────────
def bench1():
    """Horizontal bars – scaled (divide by 10 000 000)."""
    scale = lambda arr: [max(1, x // 10_000_000) for x in arr]
    y, w  = np.arange(len(ordered_ops)), 0.1
    fig, ax = plt.subplots(figsize=(12, 8))

    ax.barh(y - 2*w, scale(du_volatile_arr),   w, label='Doublets United Volatile',   color='salmon')
    ax.barh(y -   w, scale(du_nonvolatile_arr),w, label='Doublets United NonVolatile',color='red')
    ax.barh(y      , scale(ds_volatile_arr),    w, label='Doublets Split Volatile',    color='lightgreen')
    ax.barh(y +   w, scale(ds_nonvolatile_arr), w, label='Doublets Split NonVolatile', color='green')
    ax.barh(y + 2*w, scale(neo4j_non_arr),      w, label='Neo4j NonTransaction',       color='lightblue')
    ax.barh(y + 3*w, scale(neo4j_trans_arr),    w, label='Neo4j Transaction',          color='blue')

    ax.set_xlabel('Time (ns) – scaled')
    ax.set_title ('Benchmark Comparison: Neo4j vs Doublets (Rust)')
    ax.set_yticks(y); ax.set_yticklabels(ordered_ops); ax.legend()
    fig.tight_layout(); plt.savefig("bench_rust.png"); plt.close(fig)
    if DEBUG: logging.info("bench_rust.png saved.")

def bench2():
    """Horizontal bars – raw values on a log scale."""
    y, w  = np.arange(len(ordered_ops)), 0.1
    fig, ax = plt.subplots(figsize=(12, 8))

    ax.barh(y - 2*w, du_volatile_arr,   w, label='Doublets United Volatile',   color='salmon')
    ax.barh(y -   w, du_nonvolatile_arr,w, label='Doublets United NonVolatile',color='red')
    ax.barh(y      , ds_volatile_arr,    w, label='Doublets Split Volatile',    color='lightgreen')
    ax.barh(y +   w, ds_nonvolatile_arr, w, label='Doublets Split NonVolatile', color='green')
    ax.barh(y + 2*w, neo4j_non_arr,      w, label='Neo4j NonTransaction',       color='lightblue')
    ax.barh(y + 3*w, neo4j_trans_arr,    w, label='Neo4j Transaction',          color='blue')

    ax.set_xlabel('Time (ns) – log scale')
    ax.set_title ('Benchmark Comparison: Neo4j vs Doublets (Rust)')
    ax.set_yticks(y); ax.set_yticklabels(ordered_ops); ax.set_xscale('log'); ax.legend()
    fig.tight_layout(); plt.savefig("bench_rust_log_scale.png"); plt.close(fig)
    if DEBUG: logging.info("bench_rust_log_scale.png saved.")

# ─────────────────────────────────────────────────────────────────────────────
# Run
# ─────────────────────────────────────────────────────────────────────────────
print_results_markdown()
bench1()
bench2()
