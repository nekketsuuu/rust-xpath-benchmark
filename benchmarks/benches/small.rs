use std::hint::black_box;

use benchmarks::{check_timeout, skip_unsupported, write_skipped, SkippedEntry};
use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_libxml::LibxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const XML: &str = include_str!("../../fixtures/small.xml");

/// XPath 1.0 queries common to all runners.
const QUERIES_TIER1: &[(&str, &str)] = &[
    ("all_books", "//book"),
    ("fiction", "//book[@category='fiction']"),
    ("titles", "//book/title"),
    ("price_gt_12", "//book[price>12]"),
    ("count_books", "count(//book)"),
    ("first_book", "//book[1]"),
    ("contains_title", "//book[contains(title,'1984')]"),
];

/// XPath 2.0+ queries (xee-xpath, xrust, amxml only).
/// NOTE: sxd-xpath supports XPath 1.0 only and will return Err for these.
const QUERIES_TIER2: &[(&str, &str)] =
    &[("flwr_titles", "for $b in //book return string($b/title)")];

/// XPath 3.1 queries (xee-xpath only).
/// NOTE: sxd-xpath, xrust, amxml do not support XPath 3.1 simple map operator.
const QUERIES_TIER3: &[(&str, &str)] = &[("simple_map", "//book ! string(title)")];

/// Library-specific queries that fail due to bugs, not tier limitations.
/// Each entry: (query_name, library_name, reason).
const SKIP: &[(&str, &str, &str)] = &[
    ("price_gt_12", "xrust", "decimal number comparison bug"),
    (
        "contains_title",
        "amxml",
        "singleton string type error on contains()",
    ),
];

const FIXTURE: &str = "small";

macro_rules! bench_one {
    ($group:expr, $runner:expr, $name:literal, $query_name:expr, $xpath:expr, $skipped:expr) => {
        if let Some((_, _, reason)) = SKIP
            .iter()
            .find(|(q, l, _)| *q == $query_name && *l == $name)
        {
            skip_unsupported(&mut $skipped, $query_name, $name, reason);
        } else if let Some(probe_dur) = check_timeout($name, FIXTURE, $xpath) {
            eprintln!(
                "TIMEOUT: {}/{} — probe exceeded {:.2?}, skipping",
                $query_name, $name, probe_dur
            );
            $skipped.push(SkippedEntry {
                query: $query_name.to_string(),
                library: $name.to_string(),
                reason: "timeout".to_string(),
                detail: format!("probe exceeded {:?}", probe_dur),
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

fn bench_small(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(XML);
    let xee_runner = XeeXPathRunner::new(XML);
    let xrust_runner = XrustRunner::new(XML);
    let amxml_runner = AmxmlRunner::new(XML);
    let libxml_runner = LibxmlRunner::new(XML);

    let mut group = c.benchmark_group("small");
    let mut skipped = Vec::new();

    // TIER1: all runners support XPath 1.0
    for (query_name, xpath) in QUERIES_TIER1 {
        bench_one!(group, &sxd_runner, "sxd-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xrust_runner, "xrust", *query_name, xpath, skipped);
        bench_one!(group, &amxml_runner, "amxml", *query_name, xpath, skipped);
        bench_one!(group, &libxml_runner, "libxml", *query_name, xpath, skipped);
    }

    // TIER2: XPath 2.0+ (xee, xrust, amxml)
    for (query_name, xpath) in QUERIES_TIER2 {
        skip_unsupported(&mut skipped, query_name, "sxd-xpath", "XPath 1.0 only");
        skip_unsupported(&mut skipped, query_name, "libxml", "XPath 1.0 only");
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
        bench_one!(group, &xrust_runner, "xrust", *query_name, xpath, skipped);
        bench_one!(group, &amxml_runner, "amxml", *query_name, xpath, skipped);
    }

    // TIER3: XPath 3.1 (xee only)
    for (query_name, xpath) in QUERIES_TIER3 {
        skip_unsupported(&mut skipped, query_name, "sxd-xpath", "XPath 1.0 only");
        skip_unsupported(&mut skipped, query_name, "libxml", "XPath 1.0 only");
        skip_unsupported(
            &mut skipped,
            query_name,
            "xrust",
            "no XPath 3.1 simple map operator",
        );
        skip_unsupported(
            &mut skipped,
            query_name,
            "amxml",
            "no XPath 3.1 simple map operator",
        );
        bench_one!(group, &xee_runner, "xee-xpath", *query_name, xpath, skipped);
    }

    group.finish();
    write_skipped("small", &skipped);
}

criterion_group!(benches, bench_small);
criterion_main!(benches);
