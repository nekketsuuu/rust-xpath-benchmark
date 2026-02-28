use std::hint::black_box;

use benchmarks::{check_timeout, write_skipped, SkippedEntry};
use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const XML: &str = include_str!("../../fixtures/wide.xml");

/// XPath 1.0 queries on wide XML (100000 sibling <book> elements).
/// Exercises linear scan performance.
const QUERIES_TIER1: &[(&str, &str)] = &[
    ("count_books", "count(//book)"),
    ("fiction", "//book[@category='fiction']"),
    ("last_book", "//book[last()]"),
    ("book_by_id_50000", "//book[@id='50000']"),
    ("book_by_id_99999", "//book[@id='99999']"),
    ("title_lang_ja", "//book/title[@lang='ja']"),
];

/// XPath 2.0+ queries.
/// NOTE: sxd-xpath supports XPath 1.0 only and will return Err for these.
const QUERIES_TIER2: &[(&str, &str)] = &[("flwr_count", "count(for $b in //book return $b/title)")];

macro_rules! bench_one {
    ($group:expr, $runner:expr, $name:literal, $query_name:expr, $xpath:expr, $skipped:expr) => {
        if let Some(single_run) = check_timeout($runner, $xpath) {
            eprintln!(
                "TIMEOUT: {}/{} — single iteration took {:.2?}, skipping",
                $query_name, $name, single_run
            );
            $skipped.push(SkippedEntry {
                query: $query_name.to_string(),
                library: $name.to_string(),
                reason: "timeout".to_string(),
                detail: format!("single iteration took {:?}", single_run),
            });
        } else {
            $group.bench_with_input(BenchmarkId::new($query_name, $name), $xpath, |b, xpath| {
                b.iter(|| {
                    let result = $runner.evaluate(black_box(xpath));
                    black_box(result)
                })
            });
        }
    };
}

fn bench_wide(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(XML);
    let xee_runner = XeeXPathRunner::new(XML);
    let xrust_runner = XrustRunner::new(XML);
    let amxml_runner = AmxmlRunner::new(XML);

    let mut group = c.benchmark_group("wide");
    let mut skipped = Vec::new();

    // TIER1: all runners support XPath 1.0
    for (query_name, xpath) in QUERIES_TIER1 {
        bench_one!(group, &sxd_runner, "sxd-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xrust_runner, "xrust", *query_name, xpath, skipped);
        bench_one!(group, &amxml_runner, "amxml", *query_name, xpath, skipped);
    }

    // TIER2: XPath 2.0+ (xee, xrust, amxml)
    for (query_name, xpath) in QUERIES_TIER2 {
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xrust_runner, "xrust", *query_name, xpath, skipped);
        bench_one!(group, &amxml_runner, "amxml", *query_name, xpath, skipped);
    }

    group.finish();
    write_skipped("wide", &skipped);
}

criterion_group!(benches, bench_wide);
criterion_main!(benches);
