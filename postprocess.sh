#!/usr/bin/env bash
# Post-process Criterion violin plot SVGs to add median markers.
# Run this after `cargo bench` to annotate the generated plots.
#
# Usage:
#   ./postprocess.sh
#
# Each benchmark group's violin.svg will be updated in-place with
# per-library median markers (distinct shapes and colors).

set -euo pipefail

CRITERION_DIR="target/criterion"

# Build the tool once
cargo build -p violin-marker --release --quiet

for group in 01_small 02_medium 03_large 04_deep 05_wide 06_realworld_rss 07_realworld_maven 08_realworld_osm; do
    svg="${CRITERION_DIR}/${group}/report/violin.svg"
    if [ -f "$svg" ]; then
        echo "Processing ${group} ..."
        cargo run -p violin-marker --release --quiet -- "$CRITERION_DIR" "$group"
    else
        echo "Skipping ${group} (no results yet)"
    fi
done

# Rename report title
index_html="${CRITERION_DIR}/report/index.html"
if [ -f "$index_html" ]; then
    sed -i 's/Criterion\.rs Benchmark Index/Rust XPath Benchmark Index/g' "$index_html"
    echo "Updated report title."
fi

echo "Done."
