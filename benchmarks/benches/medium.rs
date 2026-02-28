use std::hint::black_box;

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

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $name:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group(concat!("medium/", $name));
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

fn bench_medium(c: &mut Criterion) {
    bench_runner!(c, XML, SxdXPathRunner, "sxd-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER2);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER1);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER2);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER1);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER2);
}

criterion_group!(benches, bench_medium);
criterion_main!(benches);
