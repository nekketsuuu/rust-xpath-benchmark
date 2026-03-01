# rust-xpath-benchmark

Benchmarks comparing five Rust XPath libraries side by side.

| Library | XPath version | Crate |
|---|---|---|
| sxd-xpath | 1.0 | [sxd-xpath](https://crates.io/crates/sxd-xpath) |
| xee-xpath | 3.1 | [xee-xpath](https://crates.io/crates/xee-xpath) |
| xrust | 2.0+ | [xrust](https://crates.io/crates/xrust) |
| amxml | 2.0+ | [amxml](https://crates.io/crates/amxml) |
| libxml | 1.0 | [libxml](https://crates.io/crates/libxml) (libxml2 wrapper) |

## Usage

```bash
# Run all benchmarks
cargo bench

# Run a single fixture
cargo bench --bench 01_small

# Add per-library markers to violin plots
./postprocess.sh
```

Results are written to `target/criterion/`. Open `report/index.html` under each group to view violin plots and other reports. The benchmark harness is [Criterion](https://bheisler.github.io/criterion.rs/book/).

## Fixtures

Synthetic data (`small`, `medium`, `large`, `deep`, `wide`) and real-world data (`rss`, `maven`, `osm`). See [fixtures/README.md](fixtures/README.md) for details.

## Notes on benchmark output

### Flat vs Linear sampling

Criterion uses a default measurement window of 5 seconds. For each case it runs increasing numbers of iterations to fit a linear regression (Linear sampling mode). When a single iteration is slow enough that Criterion cannot collect enough distinct sample points within 5 seconds, it falls back to **Flat sampling** — running a constant number of iterations per sample and reporting the mean directly, without fitting a slope.

xrust is 100–300× slower than the other libraries on most queries. As a result, many xrust cases use Flat sampling. This is visible in two ways:

- The **"Average Iteration Time" regression plot** shows a vertical cluster of points instead of a diagonal line.
- The `slope` field is `null` in the corresponding `estimates.json`.

**Violin plots and median/mean values are unaffected.** Cross-library comparisons remain valid.

### Skipped cases

Some benchmark cases are skipped for one of two reasons:

- **Timeout** — A probe binary runs `evaluate()` in a separate process with a 3-second deadline. If the probe does not finish in time, the case is skipped.
- **Unsupported** — The library does not support the required XPath version tier, or a known library bug prevents the query from succeeding.

Skipped cases are recorded in `target/criterion/<group>/skipped.json` and appear as labelled rows below the violin plot after running `./postprocess.sh`.
