use std::hint::black_box;

use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
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

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $name:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group(concat!("small/", $name));
        for (query_name, xpath) in $queries {
            group.bench_with_input(BenchmarkId::new(*query_name, ""), xpath, |b, xpath| {
                b.iter(|| {
                    let result = runner.evaluate(black_box(xpath));
                    black_box(result)
                })
            });
        }
        group.finish();
    };
}

fn bench_small(c: &mut Criterion) {
    bench_runner!(c, XML, SxdXPathRunner, "sxd-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER2);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER3);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER1);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER2);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER1);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER2);
}

criterion_group!(benches, bench_small);
criterion_main!(benches);
