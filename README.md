# rust-xpath-benchmark

Benchmarks comparing four Rust XPath libraries side by side.

| Library | XPath version | Crate |
|---|---|---|
| sxd-xpath | 1.0 | [sxd-xpath](https://crates.io/crates/sxd-xpath) |
| xee-xpath | 3.1 | [xee-xpath](https://crates.io/crates/xee-xpath) |
| xrust | 2.0+ | [xrust](https://crates.io/crates/xrust) |
| amxml | 2.0+ | [amxml](https://crates.io/crates/amxml) |

## Usage

```bash
# Run all benchmarks
cargo bench

# Run a single fixture
cargo bench --bench small

# Add per-library markers to violin plots
./postprocess.sh
```

Results are written to `target/criterion/`. Open `report/index.html` under each group to view violin plots and other reports. The benchmark harness is [Criterion](https://bheisler.github.io/criterion.rs/book/).

## Fixtures

Synthetic data (`small`, `medium`, `large`, `deep`, `wide`) and real-world data (`rss`, `maven`, `osm`). See [fixtures/README.md](fixtures/README.md) for details.
