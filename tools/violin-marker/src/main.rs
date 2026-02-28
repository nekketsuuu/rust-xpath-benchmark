//! Post-process Criterion violin plot SVGs to add median markers with
//! distinct shapes per library, making cross-library comparisons easier.
//!
//! Usage:
//!     violin-marker <criterion-base-dir> <group-name>
//!
//! Example:
//!     violin-marker target/criterion small
//!
//! This reads `<base>/<group>/report/violin.svg`, extracts Y-axis labels
//! to identify each benchmark, reads the median from each benchmark's
//! `estimates.json`, computes the X position on the plot, and inserts
//! SVG marker elements.  The result is written back to the same file.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Estimates {
    median: Statistic,
}

#[derive(Deserialize)]
struct Statistic {
    point_estimate: f64, // nanoseconds
}

/// A benchmark case that was skipped (matches benchmarks::SkippedEntry).
#[derive(Deserialize)]
struct SkippedEntry {
    query: String,
    library: String,
    reason: String,
    #[allow(dead_code)]
    detail: String,
}

// ---------------------------------------------------------------------------
// SVG axis extraction
// ---------------------------------------------------------------------------

/// Extracts the X-axis scale from the SVG content.
/// Returns (x_origin_px, px_per_unit, unit_label) by parsing the axis tick
/// labels and their pixel positions.
fn extract_x_axis(svg: &str) -> (f64, f64) {
    // Find tick labels on the X axis.  They look like:
    //   <text x="131" y="653" ...>0.0</text>
    //   <text x="258" y="653" ...>2.0</text>
    // All share the same y value (the largest y among axis labels).

    let mut ticks: Vec<(f64, f64)> = Vec::new(); // (px, value)

    // Parse <text x="..." y="653" ...>NUMBER</text> patterns.
    // The X-axis labels share a common y that is larger than Y-axis labels.
    // Strategy: collect all <text> with a numeric body, group by y, pick the
    // group whose y is largest (that's the X axis).

    struct TextLabel {
        x: f64,
        y: f64,
        body: String,
    }

    let mut labels: Vec<TextLabel> = Vec::new();
    for text_start in svg.match_indices("<text ") {
        let start = text_start.0;
        let Some(end) = svg[start..].find("</text>") else {
            continue;
        };
        let element = &svg[start..start + end + "</text>".len()];

        let x = extract_attr(element, "x");
        let y = extract_attr(element, "y");
        // Body is between > and </text>
        let Some(gt) = element.find('>') else {
            continue;
        };
        let body = element[gt + 1..element.len() - "</text>".len()].trim();

        if let (Some(x), Some(y)) = (x, y) {
            labels.push(TextLabel {
                x,
                y,
                body: body.to_string(),
            });
        }
    }

    // Group by y, find the group with the largest y that has numeric labels.
    let mut by_y: HashMap<i64, Vec<(f64, String)>> = HashMap::new();
    for l in &labels {
        let key = (l.y * 10.0) as i64; // discretize
        by_y.entry(key).or_default().push((l.x, l.body.clone()));
    }

    let mut best_y_key: Option<i64> = None;
    for (&y_key, entries) in &by_y {
        // Check if all entries are numeric
        let all_numeric = entries
            .iter()
            .all(|(_, b)| b.parse::<f64>().is_ok() && b != "Input");
        if all_numeric && entries.len() >= 2 {
            if best_y_key.is_none() || y_key > best_y_key.unwrap() {
                best_y_key = Some(y_key);
            }
        }
    }

    let y_key = best_y_key.expect("Could not find X-axis tick labels in SVG");
    for (px, body) in &by_y[&y_key] {
        if let Ok(val) = body.parse::<f64>() {
            ticks.push((*px, val));
        }
    }

    ticks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(
        ticks.len() >= 2,
        "Need at least 2 X-axis ticks, found {}",
        ticks.len()
    );

    // Linear mapping: px = origin + value * scale
    let (px0, val0) = ticks[0];
    let (px1, val1) = ticks[ticks.len() - 1];
    let px_per_unit = (px1 - px0) / (val1 - val0);
    let x_origin = px0 - val0 * px_per_unit;

    (x_origin, px_per_unit)
}

/// Extracts the unit from X-axis label text (e.g. "Average time (ms)").
/// Returns the multiplier to convert nanoseconds to the axis unit.
fn extract_time_multiplier(svg: &str) -> f64 {
    // Look for text like "Average time (ms)" or "Average time (µs)" etc.
    if svg.contains("Average time (ms)") {
        1e-6 // ns -> ms
    } else if svg.contains("Average time (µs)") || svg.contains("Average time (us)") {
        1e-3 // ns -> µs
    } else if svg.contains("Average time (s)") {
        1e-9 // ns -> s
    } else if svg.contains("Average time (ns)") {
        1.0 // ns -> ns
    } else {
        // Default to ms
        1e-6
    }
}

/// Extract Y-axis labels and their Y pixel positions.
/// Returns Vec<(label, y_px)> sorted by y ascending (top to bottom).
fn extract_y_labels(svg: &str) -> Vec<(String, f64)> {
    let mut results: Vec<(String, f64)> = Vec::new();

    // Y-axis labels have text-anchor="end" and contain "/" (group/bench path).
    for text_start in svg.match_indices("<text ") {
        let start = text_start.0;
        let Some(end) = svg[start..].find("</text>") else {
            continue;
        };
        let element = &svg[start..start + end + "</text>".len()];

        if !element.contains("text-anchor=\"end\"") {
            continue;
        }

        let Some(y) = extract_attr(element, "y") else {
            continue;
        };
        let Some(gt) = element.find('>') else {
            continue;
        };
        let body = element[gt + 1..element.len() - "</text>".len()]
            .trim()
            .to_string();

        // Y-axis labels contain the group/bench path (has "/")
        if body.contains('/') {
            results.push((body, y));
        }
    }

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    results
}

fn extract_attr(element: &str, attr_name: &str) -> Option<f64> {
    let pattern = format!("{attr_name}=\"");
    let pos = element.find(&pattern)?;
    let start = pos + pattern.len();
    let end = element[start..].find('"')? + start;
    element[start..end].parse().ok()
}

// ---------------------------------------------------------------------------
// Marker shapes
// ---------------------------------------------------------------------------

/// Library marker configuration.
struct MarkerStyle {
    color: &'static str,
    shape: MarkerShape,
}

enum MarkerShape {
    Circle,
    Diamond,
    Triangle,
    Square,
}

fn marker_for_library(lib: &str) -> MarkerStyle {
    match lib {
        "sxd-xpath" => MarkerStyle {
            color: "#E31A1C", // red
            shape: MarkerShape::Circle,
        },
        "xee-xpath" => MarkerStyle {
            color: "#1F78B4", // blue
            shape: MarkerShape::Diamond,
        },
        "xrust" => MarkerStyle {
            color: "#33A02C", // green
            shape: MarkerShape::Triangle,
        },
        "amxml" => MarkerStyle {
            color: "#FF7F00", // orange
            shape: MarkerShape::Square,
        },
        _ => MarkerStyle {
            color: "#000000",
            shape: MarkerShape::Circle,
        },
    }
}

fn render_marker(x: f64, y: f64, style: &MarkerStyle) -> String {
    let r = 4.0; // marker radius
    let stroke = "stroke=\"#000\" stroke-width=\"0.5\" opacity=\"0.9\"";
    let fill = style.color;
    match style.shape {
        MarkerShape::Circle => {
            format!("<circle cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"{r:.1}\" fill=\"{fill}\" {stroke}/>",)
        }
        MarkerShape::Diamond => {
            // Rotated square
            format!(
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{fill}\" {stroke}/>",
                x, y - r,       // top
                x + r, y,       // right
                x, y + r,       // bottom
                x - r, y,       // left
            )
        }
        MarkerShape::Triangle => {
            // Upward pointing triangle
            let h = r * 1.15; // slightly taller
            format!(
                "<polygon points=\"{:.1},{:.1} {:.1},{:.1} {:.1},{:.1}\" fill=\"{fill}\" {stroke}/>",
                x, y - h,           // top
                x + r, y + h * 0.6, // bottom right
                x - r, y + h * 0.6, // bottom left
            )
        }
        MarkerShape::Square => {
            let half = r * 0.8;
            format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{fill}\" {stroke}/>",
                x - half,
                y - half,
                half * 2.0,
                half * 2.0,
            )
        }
    }
}

fn render_legend(x: f64, y: f64, libraries: &[&str]) -> String {
    let mut parts = Vec::new();
    let line_height = 16.0;
    let marker_text_gap = 10.0;

    for (i, lib) in libraries.iter().enumerate() {
        let ly = y + i as f64 * line_height;
        let style = marker_for_library(lib);
        parts.push(render_marker(x, ly, &style));
        parts.push(format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" dy=\"0.35em\" font-family=\"sans-serif\" font-size=\"8\" fill=\"#000\">{lib}</text>",
            x + marker_text_gap, ly
        ));
    }

    parts.join("\n")
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the X-axis label ("Average time (...)") and return its (x, y)
/// pixel position so that we can place annotations relative to it.
fn find_x_axis_label_position(svg: &str) -> Option<(f64, f64)> {
    // The label element looks like:
    //   <text x="528" y="691" ...>
    //   Average time (ms)
    //   </text>
    // We search for the <text> element whose body contains "Average time".
    for text_start in svg.match_indices("<text ") {
        let start = text_start.0;
        let Some(end) = svg[start..].find("</text>") else {
            continue;
        };
        let element = &svg[start..start + end + "</text>".len()];
        let Some(gt) = element.find('>') else {
            continue;
        };
        let body = &element[gt + 1..element.len() - "</text>".len()];
        if body.contains("Average time") {
            let x = extract_attr(element, "x")?;
            let y = extract_attr(element, "y")?;
            return Some((x, y));
        }
    }
    None
}

/// Read `skipped.json` for the given group directory.
/// Returns an empty Vec if the file doesn't exist or can't be parsed.
fn read_skipped(base_dir: &Path, group_name: &str) -> Vec<SkippedEntry> {
    let path = base_dir.join(group_name).join("skipped.json");
    let Ok(data) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&data).unwrap_or_default()
}

/// Display text for a skip reason.
fn skip_reason_text(reason: &str) -> &str {
    match reason {
        "timeout" => "TIMEOUT",
        "unsupported" => "UNSUPPORTED",
        _ => "SKIPPED",
    }
}

/// Fill color for a skip reason.
fn skip_reason_color(reason: &str) -> &str {
    match reason {
        "timeout" => "#CC0000",
        "unsupported" => "#999999",
        _ => "#666666",
    }
}

/// Render SVG elements for skipped benchmark rows below the axis area.
///
/// `start_y` is the Y position for the first skipped row (caller computes
/// this so that rows appear below the "Lower is better" annotation).
///
/// Returns (svg_elements, last_row_y) where last_row_y is the Y position
/// of the last rendered row (the caller uses this to compute extra_height).
fn render_skipped_rows(
    skipped: &[SkippedEntry],
    group_name_svg: &str,
    start_y: f64,
    row_spacing: f64,
    label_x: f64,
    label_font_size: f64,
    plot_left_x: f64,
) -> (String, f64) {
    if skipped.is_empty() {
        return (String::new(), start_y);
    }

    let mut parts = Vec::new();
    let mut y = start_y;

    for entry in skipped {
        let label = format!("{}/{}/{}", group_name_svg, entry.query, entry.library);
        let reason_text = skip_reason_text(&entry.reason);
        let reason_color = skip_reason_color(&entry.reason);

        // Y-axis label (same style as Criterion's labels)
        parts.push(format!(
            "<text x=\"{:.0}\" y=\"{y:.0}\" dy=\"0.5ex\" text-anchor=\"end\" \
             font-family=\"sans-serif\" font-size=\"{label_font_size}\" \
             opacity=\"1\" fill=\"#000000\">\n{label}\n</text>",
            label_x
        ));

        // Tick mark
        parts.push(format!(
            "<polyline fill=\"none\" opacity=\"1\" stroke=\"#000000\" stroke-width=\"1\" \
             points=\"{:.0},{y:.0} {:.0},{y:.0} \"/>",
            label_x + 4.0,
            label_x + 9.0,
        ));

        // Reason text in the plot area
        parts.push(format!(
            "<text x=\"{:.0}\" y=\"{y:.0}\" dy=\"0.35em\" \
             font-family=\"sans-serif\" font-size=\"9\" font-style=\"italic\" \
             fill=\"{reason_color}\">{reason_text}</text>",
            plot_left_x + 15.0,
        ));

        y += row_spacing;
    }

    // last_row_y is the Y of the last rendered row (one row_spacing back from y)
    let last_row_y = y - row_spacing;
    (parts.join("\n"), last_row_y)
}

/// Expand the SVG's width/height attributes and viewBox by `extra_height`.
fn expand_svg_height(svg: &str, extra_height: f64) -> String {
    // Match: <svg width="960" height="726" viewBox="0 0 960 726" ...>
    // We need to update both `height="..."` and the viewBox height.
    let mut result = svg.to_string();

    // Update height="NNN" (first occurrence, in the <svg> tag)
    if let Some(pos) = result.find("height=\"") {
        let start = pos + "height=\"".len();
        if let Some(end) = result[start..].find('"') {
            let end = start + end;
            if let Ok(h) = result[start..end].parse::<f64>() {
                let new_h = h + extra_height;
                result = format!("{}{:.0}{}", &result[..start], new_h, &result[end..]);
            }
        }
    }

    // Update viewBox="0 0 960 NNN"
    if let Some(pos) = result.find("viewBox=\"") {
        let start = pos + "viewBox=\"".len();
        if let Some(end) = result[start..].find('"') {
            let end = start + end;
            let parts: Vec<&str> = result[start..end].split_whitespace().collect();
            if parts.len() == 4 {
                if let Ok(h) = parts[3].parse::<f64>() {
                    let new_vb = format!(
                        "{} {} {} {:.0}",
                        parts[0],
                        parts[1],
                        parts[2],
                        h + extra_height
                    );
                    result = format!("{}{}{}", &result[..start], new_vb, &result[end..]);
                }
            }
        }
    }

    result
}

/// Extract the current height from the SVG's `height="..."` attribute.
fn current_svg_height(svg: &str) -> f64 {
    if let Some(pos) = svg.find("height=\"") {
        let start = pos + "height=\"".len();
        if let Some(end) = svg[start..].find('"') {
            if let Ok(h) = svg[start..start + end].parse::<f64>() {
                return h;
            }
        }
    }
    0.0
}

/// Strip the group prefix from a Y-axis label, returning the
/// `<query>/<library>` suffix.
///
/// Labels have the form `<group>/<query>/<library>` where `<group>` itself
/// may contain slashes (e.g. `realworld/rss`).  The last two `/`-separated
/// segments are always query and library, so we strip everything before them.
fn strip_group_prefix(label: &str) -> &str {
    // Find the second-to-last '/'
    let bytes = label.as_bytes();
    let mut slash_count = 0;
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'/' {
            slash_count += 1;
            if slash_count == 2 {
                return &label[i + 1..];
            }
        }
    }
    // Fallback: if fewer than 2 slashes, return as-is
    label
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: violin-marker <criterion-base-dir> <group-name>");
        eprintln!("Example: violin-marker target/criterion small");
        std::process::exit(1);
    }

    let base_dir = PathBuf::from(&args[1]);
    let group_name = &args[2];

    let svg_path = base_dir.join(group_name).join("report/violin.svg");
    let svg = fs::read_to_string(&svg_path)
        .unwrap_or_else(|e| panic!("Cannot read {}: {e}", svg_path.display()));

    // Read skipped benchmarks (if any)
    let skipped = read_skipped(&base_dir, group_name);

    // Parse SVG structure
    let (x_origin, px_per_unit) = extract_x_axis(&svg);
    let time_mult = extract_time_multiplier(&svg);
    let y_labels = extract_y_labels(&svg);

    eprintln!(
        "Found {} labels, x_origin={x_origin:.1}, px_per_unit={px_per_unit:.1}, time_mult={time_mult}",
        y_labels.len()
    );

    // For each label, read the median and compute marker position
    let mut markers = Vec::new();
    let mut seen_libraries: Vec<String> = Vec::new();

    for (label, y_px) in &y_labels {
        // label is like "small/all_books/sxd-xpath" or
        // "realworld/rss/item_titles/xrust".
        //
        // The filesystem layout is:
        //   <base>/<fs_group>/<function>/<parameter>/new/estimates.json
        // where <fs_group> is the directory name passed on the command line
        // (e.g. "realworld_rss") and the label's group prefix may differ
        // (Criterion replaces "/" in group names with "_" on disk).
        //
        // The label always ends with "/<query>/<library>", so strip those
        // two trailing segments to get the SVG group prefix, then replace
        // it with the filesystem group name.
        let suffix = strip_group_prefix(label);
        let fs_label = format!("{group_name}/{suffix}");
        let estimates_path = base_dir.join(&fs_label).join("new/estimates.json");

        let median_ns = match read_median(&estimates_path) {
            Some(v) => v,
            None => {
                eprintln!("Warning: no estimates for {label}, skipping");
                continue;
            }
        };

        let median_units = median_ns * time_mult;
        let x_px = x_origin + median_units * px_per_unit;

        // Extract library name (last path component)
        let library = label.rsplit('/').next().unwrap_or(label);

        if !seen_libraries.iter().any(|l| l == library) {
            seen_libraries.push(library.to_string());
        }

        let style = marker_for_library(library);
        markers.push(render_marker(x_px, *y_px, &style));

        eprintln!(
            "  {label}: median={median_ns:.0}ns = {median_units:.4} units -> x={x_px:.1}px, y={y_px:.1}px [{library}]"
        );
    }

    if markers.is_empty() && skipped.is_empty() {
        eprintln!("No markers to add.");
        return;
    }

    // Build legend
    let lib_refs: Vec<&str> = seen_libraries.iter().map(|s| s.as_str()).collect();
    // Place legend in top-right area
    let legend_x = 850.0;
    let legend_y = 30.0;
    let legend = render_legend(legend_x, legend_y, &lib_refs);

    // Remove previously inserted markers (idempotent)
    let marker_begin = "<!-- median markers (auto-generated by violin-marker) -->";
    let marker_end = "<!-- /median markers -->";
    let svg = if let Some(start) = svg.find(marker_begin) {
        if let Some(end) = svg.find(marker_end) {
            let mut cleaned = format!("{}{}", &svg[..start], &svg[end + marker_end.len()..]);

            // Restore original SVG height if it was expanded.
            // The original height is stored in a comment like:
            //   <!-- original-svg-height: 726 -->
            let height_prefix = "<!-- original-svg-height: ";
            if let Some(hp) = svg[start..end].find(height_prefix) {
                let hp = start + hp + height_prefix.len();
                if let Some(he) = svg[hp..].find(" -->") {
                    if let Ok(orig_h) = svg[hp..hp + he].parse::<f64>() {
                        cleaned =
                            expand_svg_height(&cleaned, -(current_svg_height(&cleaned) - orig_h));
                    }
                }
            }

            cleaned
        } else {
            svg
        }
    } else {
        svg
    };

    // Record the original SVG height before expansion so we can restore it
    // on re-runs (idempotency).
    let orig_height = current_svg_height(&svg);
    let orig_height_comment = format!("<!-- original-svg-height: {orig_height:.0} -->");

    // Annotation below x-axis label: "← Lower is better"
    // Position it at the same x as the "Average time (...)" label, shifted
    // down by a fixed offset so it sits just below the axis title.
    let (ann_x, ann_y) = match find_x_axis_label_position(&svg) {
        Some((x, y)) => (x, y + 24.0),
        None => (528.0, 715.0), // fallback
    };
    let annotation = format!(
        r##"<text x="{ann_x:.0}" y="{ann_y:.0}" text-anchor="middle" font-family="sans-serif" font-size="10" fill="#666666" font-style="italic">&#x2190; Lower is better</text>"##
    );

    // Render skipped rows below the "Lower is better" annotation.
    // Visual order (top to bottom):
    //   violins → X-axis line → tick labels → "Average time" →
    //   "Lower is better" → [gap] → skipped rows → bottom
    let skipped_svg;
    let extra_height;
    if !skipped.is_empty() {
        // Determine the SVG group name from existing Y-axis labels.
        // If there are labels, strip the last two segments (query/library)
        // from the first one.  Otherwise, fall back to the filesystem group
        // name (not ideal but workable).
        let svg_group_name: String = if let Some((first_label, _)) = y_labels.first() {
            let suffix = strip_group_prefix(first_label);
            first_label[..first_label.len() - suffix.len() - 1].to_string()
        } else {
            group_name.replace('_', "/")
        };

        // Derive layout parameters from existing labels.
        let row_spacing = if y_labels.len() >= 2 {
            (y_labels[1].1 - y_labels[0].1).abs()
        } else {
            18.0
        };
        // Y-axis label x position and font size (from Criterion's defaults)
        let label_x = 121.0;
        let label_font_size = 8.065;
        // Plot area left edge (the Y-axis line)
        let plot_left_x = 130.0;

        // Start skipped rows one row_spacing below the "Lower is better"
        // annotation so they don't overlap any axis elements.
        let skipped_start_y = ann_y + row_spacing;

        let (s, last_row_y) = render_skipped_rows(
            &skipped,
            &svg_group_name,
            skipped_start_y,
            row_spacing,
            label_x,
            label_font_size,
            plot_left_x,
        );
        skipped_svg = s;

        // Expand the SVG height so the last skipped row + bottom padding fits.
        let bottom_padding = 10.0;
        let needed_height = last_row_y + bottom_padding;
        extra_height = (needed_height - orig_height).max(0.0);

        eprintln!(
            "  Adding {} skipped row(s), expanding SVG by {extra_height:.0}px",
            skipped.len()
        );
    } else {
        skipped_svg = String::new();
        extra_height = 0.0;
    }

    // Expand SVG if we added skipped rows
    let svg = if extra_height > 0.0 {
        expand_svg_height(&svg, extra_height)
    } else {
        svg
    };

    // Insert markers, legend, annotation, and skipped rows before closing </svg>
    let insert = format!(
        "\n{marker_begin}\n{orig_height_comment}\n{}\n{}\n{}\n{}\n{marker_end}\n",
        markers.join("\n"),
        legend,
        annotation,
        skipped_svg,
    );

    let new_svg = svg.replace("</svg>", &format!("{insert}</svg>"));

    fs::write(&svg_path, &new_svg)
        .unwrap_or_else(|e| panic!("Cannot write {}: {e}", svg_path.display()));

    eprintln!(
        "Wrote {} markers + legend to {}",
        markers.len(),
        svg_path.display()
    );
}

fn read_median(path: &Path) -> Option<f64> {
    let data = fs::read_to_string(path).ok()?;
    let est: Estimates = serde_json::from_str(&data).ok()?;
    Some(est.median.point_estimate)
}
