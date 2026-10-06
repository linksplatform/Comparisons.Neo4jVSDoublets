"""Tests of benchmark_report.py: python3 -m unittest discover scripts"""

import tempfile
import unittest
from pathlib import Path

import benchmark_report as report

DOUBLETS_RUN = "\n".join(
    [
        "# links: 1000",
        "# background links: 10000",
        "# cpu: AMD EPYC 7763 64-Core Processor",
        "# date: 2026-10-05",
        "# run: https://github.com/owner/repo/actions/runs/1",
    ]
    + [
        f"test {operation.replace(' ', '_')}/{implementation} ... bench: {50_000 + index:,} ns/iter (+/- 1,000)"
        for operation in report.OPERATIONS
        for index, (implementation, _, _) in enumerate(report.DOUBLETS)
    ]
)
NEO4J_RUN = "\n".join(
    [
        "# links: 1000",
        "# background links: 10000",
        "# cpu: AMD EPYC 7763 64-Core Processor",
        "# neo4j: 2026.09.0 Community",
        "# date: 2026-10-05",
        "# run: https://github.com/owner/repo/actions/runs/1",
    ]
    + [
        f"test {operation.replace(' ', '_')}/{implementation} ... bench: {time:,} ns/iter (+/- 100,000)"
        for operation in report.OPERATIONS
        for implementation, time in (
            ("Neo4j_NonTransaction", 800_000_000),
            ("Neo4j_Transaction", 400_000_000),
            ("Neo4j_Batch", 10_000_000),
        )
        if (operation, implementation) not in report.NOT_MEASURED
    ]
)


def write_results(directory, files):
    for name, text in files.items():
        (Path(directory) / name).write_text(text, encoding="utf-8")


class Formatting(unittest.TestCase):
    def test_three_significant_digits(self):
        self.assertEqual(report.significant(999.6), "1000")
        self.assertEqual(report.significant(128.4), "128")
        self.assertEqual(report.significant(1.234), "1.23")

    def test_durations_use_readable_units(self):
        self.assertEqual(report.duration(5), "5 ns")
        self.assertEqual(report.duration(79_837), "79.8 µs")
        self.assertEqual(report.duration(10_242_146), "10.2 ms")
        self.assertEqual(report.duration(1_491_658_178), "1.49 s")
        # Rounding to 1000 µs moves to the next unit.
        self.assertEqual(report.duration(999_960), "1 ms")

    def test_large_ratios_have_thousands_separators(self):
        self.assertEqual(report.ratio(36_630.1), "36,600")
        self.assertEqual(report.ratio(128.3), "128")
        self.assertEqual(report.ratio(1.234), "1.23")


class Comparison(unittest.TestCase):
    def test_faster(self):
        self.assertEqual(report.comparison((79_837, 1_000), (10_242_146, 1_099_253)), "128× faster")

    def test_slower_is_not_reported_as_a_fraction_of_faster(self):
        # The old generator printed `0.3x faster` here.
        self.assertEqual(report.comparison((30_000, 100), (10_000, 100)), "3× slower")

    def test_differences_below_the_noise_are_the_same(self):
        self.assertEqual(report.comparison((1_040, 0), (1_000, 0)), "≈ same")

    def test_overlapping_deviations_are_the_same(self):
        self.assertEqual(report.comparison((1_000, 300), (1_500, 300)), "≈ same")

    def test_the_baseline_is_the_fastest_neo4j_implementation(self):
        times = {
            ("Each All", "Neo4j_NonTransaction"): (33_524_494, 0),
            ("Each All", "Neo4j_Transaction"): (33_989_142, 0),
            ("Create", "Neo4j_NonTransaction"): (837_756_484, 0),
            ("Create", "Neo4j_Transaction"): (376_722_467, 0),
            ("Create", "Neo4j_Batch"): (10_242_146, 0),
        }
        self.assertEqual(report.baseline(times, "Each All"), "Neo4j_NonTransaction")
        self.assertEqual(report.baseline(times, "Create"), "Neo4j_Batch")


class Parsing(unittest.TestCase):
    def test_criterion_bencher_lines_and_metadata(self):
        times, metadata = report.parse(
            "# neo4j: 2026.09.0 Community\n"
            "Benchmarking Create/Neo4j_Transaction: Warming up for 1.0000 s\n"
            "test Create/Neo4j_Transaction ... bench:  44,055,505 ns/iter (+/- 5,345,991)\n"
            "test Each_All/Neo4j_Batch ... bench: 1234 ns/iter (+/- 5)\n"
        )
        self.assertEqual(
            times,
            {("Create", "Neo4j_Transaction"): (44_055_505, 5_345_991), ("Each All", "Neo4j_Batch"): (1_234, 5)},
        )
        self.assertEqual(metadata, {"neo4j": "2026.09.0 Community"})

    def test_a_failed_benchmark_is_not_published(self):
        with self.assertRaisesRegex(ValueError, "no result"):
            report.parse("test Create/Neo4j_Transaction ... FAILED\n")

    def test_unknown_benchmarks_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "unknown benchmark"):
            report.parse("test Create/Redis ... bench: 1 ns/iter (+/- 0)\n")


class Loading(unittest.TestCase):
    def test_complete_results(self):
        with tempfile.TemporaryDirectory() as directory:
            write_results(directory, {"rust-10000-doublets.txt": DOUBLETS_RUN, "rust-10000-neo4j.txt": NEO4J_RUN})
            results = report.load(directory)
        self.assertEqual(list(results), [("rust", 10000)])
        self.assertEqual(len(results[("rust", 10000)]["times"]), 8 * 7 - 1)

    def test_results_are_ordered_by_language_and_background_links(self):
        with tempfile.TemporaryDirectory() as directory:
            write_results(
                directory,
                {
                    f"{language}-{background}-{backend}.txt": run
                    for language in ("csharp", "rust")
                    for background in (100000, 10000)
                    for backend, run in (("doublets", DOUBLETS_RUN), ("neo4j", NEO4J_RUN))
                },
            )
            results = report.load(directory)
        self.assertEqual(
            list(results), [("rust", 10000), ("rust", 100000), ("csharp", 10000), ("csharp", 100000)]
        )

    def test_a_missing_result_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            lines = NEO4J_RUN.splitlines()
            write_results(
                directory,
                {"rust-10000-doublets.txt": DOUBLETS_RUN, "rust-10000-neo4j.txt": "\n".join(lines[:-1])},
            )
            with self.assertRaisesRegex(ValueError, "missing results"):
                report.load(directory)

    def test_a_missing_backend_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            write_results(directory, {"rust-10000-doublets.txt": DOUBLETS_RUN})
            with self.assertRaisesRegex(ValueError, "no neo4j results"):
                report.load(directory)

    def test_unexpected_file_names_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            write_results(directory, {"10000-doublets.txt": DOUBLETS_RUN})
            with self.assertRaisesRegex(ValueError, "expected"):
                report.load(directory)

    def test_no_results_are_rejected(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaisesRegex(ValueError, "no results"):
            report.load(directory)


class Markdown(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        write_results(
            directory.name, {"rust-10000-doublets.txt": DOUBLETS_RUN, "rust-10000-neo4j.txt": NEO4J_RUN}
        )
        self.results = report.load(directory.name)

    def test_table(self):
        lines = report.table(self.results[("rust", 10000)]["times"]).splitlines()
        self.assertEqual(
            lines[0],
            "| Operation | Doublets United Volatile | Doublets United NonVolatile | Doublets Split Volatile"
            " | Doublets Split NonVolatile | Neo4j NonTransaction | Neo4j Transaction | Neo4j Batch |",
        )
        self.assertEqual(lines[1], "| --- |" + " ---: |" * 7)
        self.assertEqual(
            lines[2],
            "| Create | 50 µs (200× faster) | 50 µs (200× faster) | 50 µs (200× faster) | 50 µs (200× faster)"
            " | 800 ms | 400 ms | **10 ms** |",
        )
        # Neo4j Batch has no Each All, so the fastest of the other two is the baseline.
        self.assertEqual(
            lines[5],
            "| Each All | 50 µs (8,000× faster) | 50 µs (8,000× faster) | 50 µs (8,000× faster)"
            " | 50 µs (8,000× faster) | 800 ms | **400 ms** | — |",
        )

    def test_provenance(self):
        self.assertEqual(
            report.provenance("rust", self.results[("rust", 10000)], 1000),
            "_Median time of one iteration with 1,000 links. Neo4j 2026.09.0 Community through neo4rs "
            f"{report.versions('rust')[0].split()[1]}; doublets {report.versions('rust')[1].split()[1]}. "
            "CPU AMD EPYC 7763 64-Core Processor, "
            "[GitHub Actions run](https://github.com/owner/repo/actions/runs/1) on 2026-10-05._",
        )

    def test_provenance_of_a_local_run_on_two_machines(self):
        result = self.results[("rust", 10000)]
        result["metadata"]["doublets"] = {"cpu": "Apple M3"}
        result["metadata"]["neo4j"] = {"cpu": "AMD EPYC 7763 64-Core Processor", "neo4j": "5.26.0"}
        self.assertIn(
            "CPU AMD EPYC 7763 64-Core Processor (Neo4j) and Apple M3 (Doublets), a local run._",
            report.provenance("rust", result, 1000),
        )

    def test_versions_come_from_the_repository(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "rust").mkdir()
            (root / "rust" / "Cargo.lock").write_text(
                '[[package]]\nname = "doublets"\nversion = "0.5.0"\n\n'
                '[[package]]\nname = "neo4rs"\nversion = "0.8.0"\n'
            )
            self.assertEqual(report.versions("rust", root), ["neo4rs 0.8.0", "doublets 0.5.0"])
            self.assertEqual(
                report.versions("csharp", root),
                ["Neo4j.Driver unknown version", "Platform.Data.Doublets unknown version"],
            )

    def test_section_has_every_language(self):
        generated = report.section(self.results, "Docs")
        self.assertIn("### Rust\n\n#### Rust: 10,000 background links, 1,000 links per iteration\n", generated)
        self.assertIn("### C#\n\n_No results yet._\n", generated)
        self.assertIn(
            "![Rust, 10,000 background links, 1,000 links per iteration, log scale]"
            "(Docs/bench_rust_log_scale_10000.png)",
            generated,
        )
        self.assertTrue(generated.startswith(report.LINT_OFF))
        self.assertTrue(generated.endswith(report.LINT_ON))

    def test_replacing_the_section_twice_gives_the_same_document(self):
        document = f"# Title\n\n{report.START_MARKER}\nold\n{report.END_MARKER}\n\n## Limitations\n"
        once = report.replace_section(document, report.section(self.results, "Docs"))
        self.assertEqual(report.replace_section(once, report.section(self.results, "Docs")), once)
        self.assertNotIn("old", once)
        self.assertTrue(once.startswith("# Title\n\n") and once.endswith("\n\n## Limitations\n"))

    def test_missing_markers_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "markers"):
            report.replace_section("# Title\n", "generated")

    def test_different_links_per_iteration_are_rejected(self):
        result = self.results[("rust", 10000)]
        result["metadata"]["neo4j"]["links"] = "100"
        with self.assertRaisesRegex(ValueError, "different or unknown links"):
            report.links_per_iteration(result)


class Charts(unittest.TestCase):
    def test_linear_and_log_charts_are_written(self):
        try:
            import matplotlib  # noqa: F401
        except ImportError:
            self.skipTest("matplotlib is not installed")
        with tempfile.TemporaryDirectory() as directory:
            write_results(directory, {"rust-10000-doublets.txt": DOUBLETS_RUN, "rust-10000-neo4j.txt": NEO4J_RUN})
            written = report.charts(report.load(directory), Path(directory) / "Docs")
            self.assertEqual([path.name for path in written], ["bench_rust_10000.png", "bench_rust_log_scale_10000.png"])
            for path in written:
                self.assertEqual(path.read_bytes()[:8], b"\x89PNG\r\n\x1a\n")


if __name__ == "__main__":
    unittest.main()
