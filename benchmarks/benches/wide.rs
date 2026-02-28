use std::hint::black_box;

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

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $name:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group(concat!("wide/", $name));
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

fn bench_wide(c: &mut Criterion) {
    bench_runner!(c, XML, SxdXPathRunner, "sxd-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER1);
    bench_runner!(c, XML, XeeXPathRunner, "xee-xpath", QUERIES_TIER2);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER1);
    bench_runner!(c, XML, XrustRunner, "xrust", QUERIES_TIER2);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER1);
    bench_runner!(c, XML, AmxmlRunner, "amxml", QUERIES_TIER2);
}

criterion_group!(benches, bench_wide);
criterion_main!(benches);
