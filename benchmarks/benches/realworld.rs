use std::hint::black_box;

use common::XPathRunner;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

const RSS_XML: &str = include_str!("../../fixtures/rss2-sample.xml");
const MAVEN_XML: &str = include_str!("../../fixtures/maven-pom.xml");
const OSM_XML: &str = include_str!("../../fixtures/osm-map.xml");

// ---------------------------------------------------------------------------
// RSS 2.0 (~3 KB, has atom: namespace)
// ---------------------------------------------------------------------------

/// XPath 1.0 queries for RSS 2.0.
/// NOTE: atom: namespace queries require namespace context; sxd-xpath and others
/// may return empty results rather than errors if namespace resolution is not configured.
const RSS_QUERIES_TIER1: &[(&str, &str)] = &[
    ("all_items", "//item"),
    ("item_titles", "//item/title"),
    ("item_links", "//item/link"),
    ("channel_title", "/rss/channel/title"),
    ("count_items", "count(//item)"),
    ("first_item", "//item[1]"),
];

// ---------------------------------------------------------------------------
// Maven pom.xml (~46 KB, two namespaces, depth 8)
// ---------------------------------------------------------------------------

/// XPath 1.0 queries for Maven pom.xml (default namespace — queries use local-name workaround).
/// NOTE: The Maven POM has a default namespace (xmlns="http://maven.apache.org/POM/4.0.0").
/// Without registering the namespace, `//dependency` will not match in namespace-aware libraries.
/// We use `*[local-name()='dependency']` for portability across all runners.
const MAVEN_QUERIES_TIER1: &[(&str, &str)] = &[
    ("all_dependencies", "//*[local-name()='dependency']"),
    ("artifact_ids", "//*[local-name()='artifactId']"),
    ("count_deps", "count(//*[local-name()='dependency'])"),
    ("group_id", "//*[local-name()='groupId']"),
    ("plugins", "//*[local-name()='plugin']"),
    (
        "project_version",
        "//*[local-name()='project']/*[local-name()='version']",
    ),
];

// ---------------------------------------------------------------------------
// OpenStreetMap (~1.75 MB, no namespace, attribute-heavy)
// ---------------------------------------------------------------------------

/// XPath 1.0 queries for OSM XML.
/// Data: © OpenStreetMap contributors, ODbL 1.0
const OSM_QUERIES_TIER1: &[(&str, &str)] = &[
    ("count_nodes", "count(//node)"),
    ("count_ways", "count(//way)"),
    ("tagged_nodes", "//node[tag]"),
    ("highway_nodes", "//node[tag[@k='highway']]"),
    ("bus_stops", "//node[tag[@k='highway' and @v='bus_stop']]"),
    ("primary_roads", "//way[tag[@k='highway' and @v='primary']]"),
    ("named_ways", "//way[tag[@k='name']]"),
];

// ---------------------------------------------------------------------------
// Benchmark groups
// ---------------------------------------------------------------------------

macro_rules! bench_runner {
    ($c:expr, $xml:expr, $runner_ty:ty, $group:literal, $queries:expr) => {
        let runner = <$runner_ty>::new($xml);
        let mut group = $c.benchmark_group($group);
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

fn bench_rss(c: &mut Criterion) {
    bench_runner!(
        c,
        RSS_XML,
        SxdXPathRunner,
        "realworld/rss/sxd-xpath",
        RSS_QUERIES_TIER1
    );
    bench_runner!(
        c,
        RSS_XML,
        XeeXPathRunner,
        "realworld/rss/xee-xpath",
        RSS_QUERIES_TIER1
    );
    bench_runner!(
        c,
        RSS_XML,
        XrustRunner,
        "realworld/rss/xrust",
        RSS_QUERIES_TIER1
    );
    bench_runner!(
        c,
        RSS_XML,
        AmxmlRunner,
        "realworld/rss/amxml",
        RSS_QUERIES_TIER1
    );
}

fn bench_maven(c: &mut Criterion) {
    bench_runner!(
        c,
        MAVEN_XML,
        SxdXPathRunner,
        "realworld/maven/sxd-xpath",
        MAVEN_QUERIES_TIER1
    );
    bench_runner!(
        c,
        MAVEN_XML,
        XeeXPathRunner,
        "realworld/maven/xee-xpath",
        MAVEN_QUERIES_TIER1
    );
    bench_runner!(
        c,
        MAVEN_XML,
        XrustRunner,
        "realworld/maven/xrust",
        MAVEN_QUERIES_TIER1
    );
    bench_runner!(
        c,
        MAVEN_XML,
        AmxmlRunner,
        "realworld/maven/amxml",
        MAVEN_QUERIES_TIER1
    );
}

fn bench_osm(c: &mut Criterion) {
    bench_runner!(
        c,
        OSM_XML,
        SxdXPathRunner,
        "realworld/osm/sxd-xpath",
        OSM_QUERIES_TIER1
    );
    bench_runner!(
        c,
        OSM_XML,
        XeeXPathRunner,
        "realworld/osm/xee-xpath",
        OSM_QUERIES_TIER1
    );
    bench_runner!(
        c,
        OSM_XML,
        XrustRunner,
        "realworld/osm/xrust",
        OSM_QUERIES_TIER1
    );
    bench_runner!(
        c,
        OSM_XML,
        AmxmlRunner,
        "realworld/osm/amxml",
        OSM_QUERIES_TIER1
    );
}

criterion_group!(benches, bench_rss, bench_maven, bench_osm);
criterion_main!(benches);
