use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use wait_timeout::ChildExt;

/// Wall-clock timeout for the probe process.  If the probe (XML parse +
/// evaluate) does not finish within this duration, the case is skipped.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// A benchmark case that was skipped rather than measured.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedEntry {
    pub query: String,
    pub library: String,
    pub reason: String,
    pub detail: String,
}

/// Locate the probe binary.
///
/// The probe binary is built as part of the `benchmarks` crate.  When
/// benchmarks run via `cargo bench`, the binary is in the same target
/// directory.  We walk up from `CARGO_MANIFEST_DIR` to find the workspace
/// root, then look for the binary under `target/release/probe` or
/// `target/debug/probe`.
fn probe_bin_path() -> std::path::PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Find the workspace root by looking for a Cargo.toml with [workspace].
    let workspace_root = manifest_dir
        .ancestors()
        .skip(1)
        .find(|ancestor| {
            let toml = ancestor.join("Cargo.toml");
            std::fs::read_to_string(&toml)
                .map(|c| c.contains("[workspace]"))
                .unwrap_or(false)
        })
        .unwrap_or(manifest_dir);

    // Prefer release build (cargo bench uses release by default).
    let release = workspace_root.join("target/release/probe");
    if release.exists() {
        return release;
    }
    let debug = workspace_root.join("target/debug/probe");
    if debug.exists() {
        return debug;
    }

    // Fallback: assume it is on PATH.
    "probe".into()
}

/// Spawn the probe binary and wait with a timeout.
///
/// Returns `None` if the probe completes successfully within
/// [`PROBE_TIMEOUT`].  Returns `Some(PROBE_TIMEOUT)` if it times out.
///
/// # Panics
///
/// Panics if the probe exits with a non-zero status (evaluate error),
/// which indicates a SKIP list omission.
pub fn check_timeout(library: &str, fixture: &str, xpath: &str) -> Option<Duration> {
    let probe = probe_bin_path();
    let mut child = Command::new(&probe)
        .args([library, fixture, xpath])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("Failed to spawn probe binary {}: {e}", probe.display()));

    match child.wait_timeout(PROBE_TIMEOUT) {
        Ok(Some(status)) => {
            if status.success() {
                // Probe finished in time and evaluate() succeeded.
                None
            } else {
                // Probe finished but evaluate() returned an error.
                // Read stderr for details.
                let stderr = child
                    .stderr
                    .take()
                    .and_then(|mut s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        s.read_to_string(&mut buf).ok()?;
                        Some(buf)
                    })
                    .unwrap_or_default();
                panic!(
                    "Probe {library}/{fixture} failed (exit {}):\n  xpath: {xpath}\n  {stderr}",
                    status.code().unwrap_or(-1),
                );
            }
        }
        Ok(None) => {
            // Timeout: the probe did not finish in time.  Kill it.
            child.kill().ok();
            child.wait().ok();
            Some(PROBE_TIMEOUT)
        }
        Err(e) => {
            panic!("Failed to wait on probe process: {e}");
        }
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
