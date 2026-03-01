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

# Rename titles across all report pages. Idempotent (no-op if already applied).
find "$CRITERION_DIR" -name 'index.html' -exec \
    sed -i 's/ - Criterion\.rs<\/title>/ - Rust XPath Benchmark (Feb 2026)<\/title>/g' {} +

# Add repo link to footer on all report pages. Idempotent.
find "$CRITERION_DIR" -name 'index.html' -exec \
    sed -i '/<!-- FOOTER-REPO -->/d' {} +
find "$CRITERION_DIR" -name 'index.html' -exec \
    sed -i 's|library in Rust\.</p>|library in Rust.</p>\n        <!-- FOOTER-REPO --><p><a href="https://github.com/nekketsuuu/rust-xpath-benchmark">https://github.com/nekketsuuu/rust-xpath-benchmark</a></p>|' {} +
# Expand footer height to fit the extra line. Idempotent.
find "$CRITERION_DIR" -name 'index.html' -exec \
    sed -i 's/height: 40px;/height: auto; padding: 8px 0;/' {} +

# Customize the top-level report index
index_html="${CRITERION_DIR}/report/index.html"
if [ -f "$index_html" ]; then
    sed -i 's/Criterion\.rs Benchmark Index/Rust XPath Benchmark Index/g' "$index_html"
    # Add date subtitle under the h2 title. Idempotent.
    sed -i '/<!-- DATE-SUBTITLE -->/d' "$index_html"
    sed -i 's|<h2>Rust XPath Benchmark Index</h2>|<h2>Rust XPath Benchmark Index</h2>\n        <!-- DATE-SUBTITLE --><p style="font-size:20px;font-weight:300;color:#666;margin-top:-16px">Feb 2026</p>|' "$index_html"

    # Inject library comparison table (static, matching README.md). Idempotent.
    sed -i '/<!-- LIB-BEGIN -->/,/<!-- LIB-END -->/d' "$index_html"
    lib_tmp="$(mktemp)"
    cat > "$lib_tmp" <<'LIBHTML'
        <!-- LIB-BEGIN -->
        <h3>Libraries</h3>
        <table>
            <tr><th>Library</th><th>Supported XPath Version</th></tr>
            <tr><td><a href="https://crates.io/crates/amxml">amxml</a> 0.5.3</td><td>1.0 + partial 2.0/3.0/3.1</td></tr>
            <tr><td><a href="https://crates.io/crates/libxml">libxml</a> 0.3.8</td><td>1.0</td></tr>
            <tr><td><a href="https://crates.io/crates/sxd-xpath">sxd-xpath</a> 0.4.2</td><td>1.0</td></tr>
            <tr><td><a href="https://crates.io/crates/xee-xpath">xee-xpath</a> 0.1.5</td><td>3.1</td></tr>
            <tr><td><a href="https://crates.io/crates/xrust">xrust</a> 2.0.3</td><td>1.0 + partial 2.0/3.0</td></tr>
        </table>
        <!-- LIB-END -->
LIBHTML
    sed -i '/See individual benchmark pages below for more details\./r '"$lib_tmp" "$index_html"
    rm -f "$lib_tmp"

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
    sed -i '/<!-- LIB-END -->/r '"$env_tmp" "$index_html"
    rm -f "$env_tmp"

    # Inject "Results" heading between info tables and benchmark results. Idempotent.
    sed -i '/<!-- RESULTS-HEADING -->/d' "$index_html"
    sed -i '/<!-- ENV-END -->/a\        <!-- RESULTS-HEADING --><h3>Results</h3>' "$index_html"

    # Fix layout: widen .body and make result tables horizontally scrollable.
    # Idempotent: the CSS replacement is a no-op if already applied, and the
    # table wrapping uses N (read next line) so the substitution pattern won't
    # match once the <div> is already present.
    sed -i 's/width: 960px;/max-width: 95%;/' "$index_html"
    sed -i '/<li>/{N;s|<li>\n\(\s*<table>\)|<li><div style="overflow-x:auto">\n\1|;}' "$index_html"
    sed -i '/<\/table>/{N;s|</table>\n\(\s*</li>\)|</table></div>\n\1|;}' "$index_html"

    echo "Updated report title and injected environment info."
fi

# Add "Index" navigation link to all sub-pages. Idempotent.
while IFS= read -r -d '' page; do
    # Skip the top-level report index itself
    [ "$page" = "${CRITERION_DIR}/report/index.html" ] && continue
    # Strip marker from previous runs
    sed -i '/<!-- NAV-INDEX -->/d' "$page"
    # Compute relative path: from <subdir>/report/index.html back to report/index.html
    rel="${page#"${CRITERION_DIR}/"}"          # e.g. 01_small/report/index.html
    dir="$(dirname "$rel")"                     # e.g. 01_small/report
    depth="$(echo "$dir" | tr '/' '\n' | wc -l)" # e.g. 2
    prefix="$(printf '../%.0s' $(seq 1 "$depth"))" # e.g. ../../
    link="${prefix}report/index.html"
    sed -i "s|<div class=\"body\">|<div class=\"body\">\n        <!-- NAV-INDEX --><p><a href=\"${link}\">Rust XPath Benchmark Index</a></p>|" "$page"
done < <(find "$CRITERION_DIR" -name 'index.html' -print0)
echo "Added index links to sub-pages."

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
