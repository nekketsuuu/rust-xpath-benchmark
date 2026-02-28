use std::hint::black_box;

use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const XML: &str = include_str!("../../fixtures/large.xml");

/// XPath 1.0 queries common to all runners (50000 books).
/// Avoid returning the entire nodeset for large result sets — use count() or position filters.
const QUERIES_TIER1: &[(&str, &str)] = &[
    ("count_books", "count(//book)"),
    ("fiction", "//book[@category='fiction']"),
    ("price_gt_40", "//book[price>40]"),
    ("first_book", "//book[1]"),
    ("last_book", "//book[last()]"),
    ("book_1000", "//book[1000]"),
    ("contains_title", "//book[contains(title,'1000')]"),
];

/// XPath 2.0+ queries.
/// NOTE: sxd-xpath supports XPath 1.0 only and will return Err for these.
const QUERIES_TIER2: &[(&str, &str)] = &[("flwr_count", "count(for $b in //book return $b/title)")];

macro_rules! bench_one {
    ($group:expr, $runner:expr, $name:literal, $xpath:expr) => {
        $group.bench_with_input(BenchmarkId::from_parameter($name), $xpath, |b, xpath| {
            b.iter(|| {
                let result = $runner.evaluate(black_box(xpath));
                black_box(result)
            })
        });
    };
}

fn bench_large(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(XML);
    let xee_runner = XeeXPathRunner::new(XML);
    let xrust_runner = XrustRunner::new(XML);
    let amxml_runner = AmxmlRunner::new(XML);

    // TIER1: all runners support XPath 1.0
    for (query_name, xpath) in QUERIES_TIER1 {
        let mut group = c.benchmark_group(format!("large/{query_name}"));
        bench_one!(group, &sxd_runner, "sxd-xpath", xpath);
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }

    // TIER2: XPath 2.0+ (xee, xrust, amxml)
    for (query_name, xpath) in QUERIES_TIER2 {
        let mut group = c.benchmark_group(format!("large/{query_name}"));
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }
}

criterion_group!(benches, bench_large);
criterion_main!(benches);
