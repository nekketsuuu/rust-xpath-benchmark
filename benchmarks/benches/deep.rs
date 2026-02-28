use std::hint::black_box;

use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const XML: &str = include_str!("../../fixtures/deep-nest.xml");

/// XPath 1.0 queries on deeply nested XML (200 levels, leaf has a <book> element).
const QUERIES_TIER1: &[(&str, &str)] = &[
    ("descendant_book", "//book"),
    ("descendant_title", "//title"),
    ("deep_level_100", "//level[@n='100']"),
    ("deep_level_200", "//level[@n='200']"),
    ("ancestor_count", "count(//book/ancestor::*)"),
    ("book_price", "//book/price"),
];

/// XPath 2.0+ queries.
/// NOTE: sxd-xpath supports XPath 1.0 only and will return Err for these.
const QUERIES_TIER2: &[(&str, &str)] = &[("flwr_deep", "for $l in //level return string($l/@n)")];

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

fn bench_deep(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(XML);
    let xee_runner = XeeXPathRunner::new(XML);
    let xrust_runner = XrustRunner::new(XML);
    let amxml_runner = AmxmlRunner::new(XML);

    // TIER1: all runners support XPath 1.0
    for (query_name, xpath) in QUERIES_TIER1 {
        let mut group = c.benchmark_group(format!("deep/{query_name}"));
        bench_one!(group, &sxd_runner, "sxd-xpath", xpath);
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }

    // TIER2: XPath 2.0+ (xee, xrust, amxml)
    for (query_name, xpath) in QUERIES_TIER2 {
        let mut group = c.benchmark_group(format!("deep/{query_name}"));
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }
}

criterion_group!(benches, bench_deep);
criterion_main!(benches);
