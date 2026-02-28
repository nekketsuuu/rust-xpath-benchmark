# CLAUDE.md

## Conventions

- Prefer shell scripts or Rust for tooling. Do not use Python.

## Benchmark design decisions

- XML parsing (`XPathRunner::new`) is excluded from timing. Only `evaluate()` is measured.
- All four libraries run in a single binary. They do not measurably interfere with each other.
- Each fixture has one `benchmark_group()` call (e.g. `"small"`), not one per library.
- `BenchmarkId::new(query_name, library_name)` — query first, library second. This makes same-query cross-library comparisons adjacent in violin plots.
- `realworld.rs` has three separate groups (`realworld/rss`, `realworld/maven`, `realworld/osm`) intentionally. Do not merge them.
- Queries are tiered by XPath version support:
  - Tier 1 (XPath 1.0): all four libraries
  - Tier 2 (XPath 2.0+): xee-xpath, xrust, amxml (not sxd-xpath)
  - Tier 3 (XPath 3.1): xee-xpath only

## violin-marker tool

`tools/violin-marker` post-processes Criterion violin SVGs to add median markers. It is idempotent — re-running strips previous markers before inserting new ones. Run via `./postprocess.sh`, which skips groups without results.

It also reads `skipped.json` (if present) and appends labelled rows for skipped cases below the existing violin plot, expanding the SVG height as needed.

## Timeout / skipped benchmarks

- Before each benchmark case, a single probe iteration is run. If `iteration_time * 100 > 300s`, the case is skipped (not registered with Criterion at all).
- Skipped cases are recorded in `target/criterion/<group>/skipped.json` with fields: `query`, `library`, `reason`, `detail`.
- `reason` is currently `"timeout"` but is designed to support future values like `"unsupported"`.
- The `skipped.json` file is overwritten on each benchmark run (not appended).
- Common timeout logic lives in `benchmarks/src/lib.rs` (`check_timeout`, `write_skipped`, `SkippedEntry`).
- violin-marker reads `skipped.json` and appends labelled rows below the existing violin plot (with SVG height expansion). This is idempotent.

## Criterion quirks

- It's a bit hard to customize the violin plots with Criterion. The violin-marker tool works around this by overlaying shaped/colored markers.
- Criterion has no cross-group comparison. Comparisons only work within a single `benchmark_group()` call.
- Criterion has no built-in timeout or skip mechanism ([issue #838](https://github.com/bheisler/criterion.rs/issues/838)). The probe-based approach above is our workaround.
