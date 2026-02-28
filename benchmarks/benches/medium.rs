use std::hint::black_box;

use benchmarks::{check_timeout, skip_unsupported, write_skipped, SkippedEntry};
use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const XML: &str = include_str!("../../fixtures/medium.xml");

/// XPath 1.0 queries common to all runners (1000 books).
const QUERIES_TIER1: &[(&str, &str)] = &[
    ("all_books", "//book"),
    ("fiction", "//book[@category='fiction']"),
    ("titles", "//book/title"),
    ("price_gt_30", "//book[price>30]"),
    ("count_books", "count(//book)"),
    ("first_book", "//book[1]"),
    ("last_book", "//book[last()]"),
    ("contains_title", "//book[contains(title,'100')]"),
];

/// XPath 2.0+ queries.
/// NOTE: sxd-xpath supports XPath 1.0 only and will return Err for these.
const QUERIES_TIER2: &[(&str, &str)] =
    &[("flwr_titles", "for $b in //book return string($b/title)")];

/// Library-specific queries that fail due to bugs, not tier limitations.
/// Each entry: (query_name, library_name, reason).
const SKIP: &[(&str, &str, &str)] = &[
    ("price_gt_30", "xrust", "decimal number comparison bug"),
    (
        "contains_title",
        "amxml",
        "singleton string type error on contains()",
    ),
];

macro_rules! bench_one {
    ($group:expr, $runner:expr, $name:literal, $query_name:expr, $xpath:expr, $skipped:expr) => {
        if let Some((_, _, reason)) = SKIP
            .iter()
            .find(|(q, l, _)| *q == $query_name && *l == $name)
        {
            skip_unsupported(&mut $skipped, $query_name, $name, reason);
        } else if let Some(single_run) = check_timeout($runner, $xpath) {
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

fn bench_medium(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(XML);
    let xee_runner = XeeXPathRunner::new(XML);
    let xrust_runner = XrustRunner::new(XML);
    let amxml_runner = AmxmlRunner::new(XML);

    let mut group = c.benchmark_group("medium");
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
        skip_unsupported(&mut skipped, query_name, "sxd-xpath", "XPath 1.0 only");
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xrust_runner, "xrust", *query_name, xpath, skipped);
        bench_one!(group, &amxml_runner, "amxml", *query_name, xpath, skipped);
    }

    group.finish();
    write_skipped("medium", &skipped);
}

criterion_group!(benches, bench_medium);
criterion_main!(benches);
