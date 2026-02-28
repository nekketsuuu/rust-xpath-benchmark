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

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $name:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group(concat!("deep/", $name));
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

fn bench_deep(c: &mut Criterion) {
    bench_runner!(c, XML, SxdXPathRunner, "sxd-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER2);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER1);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER2);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER1);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER2);
}

criterion_group!(benches, bench_deep);
criterion_main!(benches);
