use std::hint::black_box;
use std::path::Path;
use std::time::{Duration, Instant};

use common::XPathRunner;
use serde::{Deserialize, Serialize};

/// Budget for a single benchmark case: if `single_iteration_time * SAMPLE_COUNT`
/// exceeds this, the case is skipped as a timeout.
pub const TIMEOUT_BUDGET: Duration = Duration::from_secs(300);

/// Expected number of samples Criterion will collect (the default).
pub const SAMPLE_COUNT: u32 = 100;

/// A benchmark case that was skipped rather than measured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedEntry {
    pub query: String,
    pub library: String,
    pub reason: String,
    pub detail: String,
}

/// Run a single probe iteration of `evaluate()` and return its duration if
/// the estimated total time for [`SAMPLE_COUNT`] samples would exceed
/// [`TIMEOUT_BUDGET`].  Returns `None` if the benchmark is fast enough.
///
/// # Panics
///
/// Panics if `evaluate()` returns `Err`.  All benchmark cases are expected to
/// succeed; unsupported library/query combinations should be recorded via
/// [`skip_unsupported`] *before* calling this function.
pub fn check_timeout<R: XPathRunner>(runner: &R, xpath: &str) -> Option<Duration> {
    let start = Instant::now();
    let result = black_box(runner.evaluate(black_box(xpath)));
    let elapsed = start.elapsed();
    result.unwrap_or_else(|e| panic!("Unexpected evaluate() error: {e}"));
    if elapsed * SAMPLE_COUNT > TIMEOUT_BUDGET {
        Some(elapsed)
    } else {
        None
    }
}

/// Record a benchmark case as skipped because the library does not support
/// the query's XPath version.
pub fn skip_unsupported(
    skipped: &mut Vec<SkippedEntry>,
    query_name: &str,
    library: &str,
    detail: &str,
) {
    skipped.push(SkippedEntry {
        query: query_name.to_string(),
        library: library.to_string(),
        reason: "unsupported".to_string(),
        detail: detail.to_string(),
    });
}

/// Locate the Criterion output directory.
///
/// Criterion resolves the target directory via `cargo metadata`, which for
/// workspace builds returns the workspace-root `target/`.  We approximate
/// this by walking up from `CARGO_MANIFEST_DIR` (the benchmarks crate root)
/// until we find a `Cargo.toml` that contains `[workspace]`.
fn criterion_output_dir() -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    // In a workspace the parent of the member crate is typically the
    // workspace root.  Fall back to `./target/criterion` if not found.
    for ancestor in manifest_dir.ancestors().skip(1) {
        let toml = ancestor.join("Cargo.toml");
        if let Ok(contents) = std::fs::read_to_string(&toml) {
            if contents.contains("[workspace]") {
                return ancestor.join("target/criterion");
            }
        }
    }
    Path::new("target/criterion").to_path_buf()
}

/// Write `skipped.json` into the Criterion output directory for `group_name`.
///
/// `group_name` should be the Criterion group name (may contain `/`).
/// Slashes are replaced with `_` to match the filesystem layout that
/// Criterion uses.
pub fn write_skipped(group_name: &str, entries: &[SkippedEntry]) {
    let fs_name = group_name.replace('/', "_");
    let dir = criterion_output_dir().join(fs_name);
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("skipped.json");
    let json = serde_json::to_string_pretty(entries).expect("failed to serialize skipped.json");
    std::fs::write(path, json).expect("failed to write skipped.json");
}
