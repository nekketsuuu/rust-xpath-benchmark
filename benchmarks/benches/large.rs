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

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $name:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group(concat!("large/", $name));
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

fn bench_large(c: &mut Criterion) {
    bench_runner!(c, XML, SxdXPathRunner, "sxd-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER2);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER1);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER2);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER1);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER2);
}

criterion_group!(benches, bench_large);
criterion_main!(benches);
