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

for group in small medium large deep wide realworld/rss realworld/maven realworld/osm; do
    svg="${CRITERION_DIR}/${group}/report/violin.svg"
    if [ -f "$svg" ]; then
        echo "Processing ${group} ..."
        cargo run -p violin-marker --release --quiet -- "$CRITERION_DIR" "$group"
    else
        echo "Skipping ${group} (no results yet)"
    fi
done

echo "Done."
