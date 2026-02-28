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

/// Run a single iteration of `evaluate()` and return its duration if the
/// estimated total time for [`SAMPLE_COUNT`] samples would exceed
/// [`TIMEOUT_BUDGET`].  Returns `None` if the benchmark is fast enough.
pub fn check_timeout<R: XPathRunner>(runner: &R, xpath: &str) -> Option<Duration> {
    let start = Instant::now();
    let _ = black_box(runner.evaluate(black_box(xpath)));
    let elapsed = start.elapsed();
    if elapsed * SAMPLE_COUNT > TIMEOUT_BUDGET {
        Some(elapsed)
    } else {
        None
    }
}

/// Write `skipped.json` into the Criterion output directory for `group_name`.
///
/// `group_name` should be the Criterion group name (may contain `/`).
/// Slashes are replaced with `_` to match the filesystem layout that
/// Criterion uses.
pub fn write_skipped(group_name: &str, entries: &[SkippedEntry]) {
    let fs_name = group_name.replace('/', "_");
    let dir = Path::new("target/criterion").join(fs_name);
    std::fs::create_dir_all(&dir).ok();
    let path = dir.join("skipped.json");
    let json = serde_json::to_string_pretty(entries).expect("failed to serialize skipped.json");
    std::fs::write(path, json).expect("failed to write skipped.json");
}
