# CLAUDE.md

## Conventions

- Prefer shell scripts or Rust for tooling. Do not use Python.

## Benchmark design decisions

- XML parsing (`XPathRunner::new`) is excluded from timing. Only `evaluate()` is measured.
- All five libraries run in a single binary. They do not measurably interfere with each other.
- Each fixture has one `benchmark_group()` call (e.g. `"01_small"`), not one per library.
- `BenchmarkId::new(query_name, library_name)` — query first, library second. This makes same-query cross-library comparisons adjacent in violin plots.
- `realworld.rs` has three separate groups (`06_realworld/rss`, `07_realworld/maven`, `08_realworld/osm`) intentionally. Do not merge them.
- Queries are tiered by XPath version support:
  - Tier 1 (XPath 1.0): all five libraries
  - Tier 2 (XPath 2.0+): xee-xpath, xrust, amxml (not sxd-xpath, libxml)
  - Tier 3 (XPath 3.1): xee-xpath only

## violin-marker tool

`tools/violin-marker` post-processes Criterion violin SVGs to add median markers. It is idempotent — re-running strips previous markers before inserting new ones. Run via `./postprocess.sh`, which skips groups without results.

It also reads `skipped.json` (if present) and appends labelled rows for skipped cases below the existing violin plot, expanding the SVG height as needed.

## Timeout / skipped benchmarks

- Before each benchmark case, a probe binary (`benchmarks/src/bin/probe.rs`) is spawned in a separate process. If the probe (XML parse + `evaluate()`) does not finish within 3 seconds (`PROBE_TIMEOUT`), the process is killed and the case is skipped.
- The probe binary embeds all fixture XML via `include_str!` and is self-contained. It accepts `<library> <fixture> <xpath>` arguments.
- Because the probe runs in a separate process, slow cases are truly terminated (SIGKILL) rather than blocking the benchmark binary. This is necessary because some runners use `Rc` (`!Send`) and cannot be moved to a background thread.
- The timeout includes XML parse time, not just `evaluate()`. This is intentionally coarse — the goal is to skip cases that would take too long, not to measure precisely.
- If the probe exits with a non-zero status (evaluate error), `check_timeout` panics — this indicates a SKIP list omission.
- Skipped cases are recorded in `target/criterion/<group>/skipped.json` with fields: `query`, `library`, `reason`, `detail`.
- `reason` is `"timeout"` or `"unsupported"` (designed to support future values too).
- Unsupported cases are declared statically via `skip_unsupported()` based on TIER classification, not by parsing error messages from `evaluate()`.
- Each bench file has a `SKIP` constant listing library-specific query failures (bugs, not tier limitations). The `bench_one!` macro checks `SKIP` before calling `check_timeout`, so skipped cases never run a probe. When adding a new query, run `cargo bench --bench <bench-name> -- 'NOMATCH'` to quickly verify all probes pass without measuring anything. Bench binary names have numeric prefixes (e.g. `01_small`, `02_medium`) for execution ordering.
- The `skipped.json` file is overwritten on each benchmark run (not appended).
- Common skip/timeout logic lives in `benchmarks/src/lib.rs` (`check_timeout`, `skip_unsupported`, `write_skipped`, `SkippedEntry`).
- violin-marker reads `skipped.json` and appends labelled rows below the existing violin plot (with SVG height expansion). This is idempotent.

## Criterion quirks

- It's a bit hard to customize the violin plots with Criterion. The violin-marker tool works around this by overlaying shaped/colored markers.
- Criterion has no cross-group comparison. Comparisons only work within a single `benchmark_group()` call.
- Criterion has no built-in timeout or skip mechanism ([issue #838](https://github.com/bheisler/criterion.rs/issues/838)). The probe-based approach above is our workaround.
