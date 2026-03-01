#!/usr/bin/env bash
# Post-process Criterion benchmark results.
# Run this after `cargo bench` to:
#   1. Add per-library median markers to violin plot SVGs
#   2. Rename the report title
#   3. Copy the report to docs/ for GitHub Pages
#
# Usage:
#   ./postprocess.sh

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

# Rename report title and inject environment info
index_html="${CRITERION_DIR}/report/index.html"
if [ -f "$index_html" ]; then
    sed -i 's/Criterion\.rs Benchmark Index/Rust XPath Benchmark Index/g' "$index_html"

    # Inject environment info (static, matching README.md). Idempotent:
    # strip previous insertion (with or without markers) before inserting.
    sed -i '/<!-- ENV-BEGIN -->/,/<!-- ENV-END -->/d' "$index_html"
    sed -i '/<h3>Environment<\/h3>/,/<\/table>/d' "$index_html"
    env_tmp="$(mktemp)"
    cat > "$env_tmp" <<'ENVHTML'
        <!-- ENV-BEGIN -->
        <h3>Environment</h3>
        <table>
            <tr><td><b>CPU</b></td><td>Intel Core i7-8700 @ 3.20GHz, 6 cores / 12 threads</td></tr>
            <tr><td><b>RAM</b></td><td>64 GiB host, 31 GiB available in WSL 2</td></tr>
            <tr><td><b>OS</b></td><td>Windows 11 Home 25H2, WSL 2 + Ubuntu 24.04.3 LTS</td></tr>
            <tr><td><b>Rust</b></td><td>rustc 1.93.1 (01f6ddf75 2026-02-11)</td></tr>
            <tr><td><b>libxml2</b></td><td>2.9.14</td></tr>
            <tr><td><b>Criterion</b></td><td>0.5.1</td></tr>
        </table>
        <!-- ENV-END -->
ENVHTML
    sed -i '/See individual benchmark pages below for more details\./r '"$env_tmp" "$index_html"
    rm -f "$env_tmp"

    # Fix layout: widen .body and make result tables horizontally scrollable.
    # Idempotent: the CSS replacement is a no-op if already applied, and the
    # table wrapping uses N (read next line) so the substitution pattern won't
    # match once the <div> is already present.
    sed -i 's/width: 960px;/max-width: 95%;/' "$index_html"
    sed -i '/<li>/{N;s|<li>\n\(\s*<table>\)|<li><div style="overflow-x:auto">\n\1|;}' "$index_html"
    sed -i '/<\/table>/{N;s|</table>\n\(\s*</li>\)|</table></div>\n\1|;}' "$index_html"

    echo "Updated report title and injected environment info."
fi

# Copy to docs/ for GitHub Pages
rm -rf docs
cp -r "$CRITERION_DIR" docs
echo "Copied report to docs/."

# Create docs/index.html as a redirect to report/
cat > docs/index.html <<'HTML'
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Rust XPath Benchmark</title>
</head>
<body>
  <p><a href="./report/">Rust XPath Benchmark Report</a></p>
</body>
</html>
HTML
echo "Created docs/index.html."

echo "Done."
