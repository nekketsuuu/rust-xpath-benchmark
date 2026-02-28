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
/// Data: (c) OpenStreetMap contributors, ODbL 1.0
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
// Benchmark helpers
// ---------------------------------------------------------------------------

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

fn bench_rss(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(RSS_XML);
    let xee_runner = XeeXPathRunner::new(RSS_XML);
    let xrust_runner = XrustRunner::new(RSS_XML);
    let amxml_runner = AmxmlRunner::new(RSS_XML);

    for (query_name, xpath) in RSS_QUERIES_TIER1 {
        let mut group = c.benchmark_group(format!("realworld/rss/{query_name}"));
        bench_one!(group, &sxd_runner, "sxd-xpath", xpath);
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }
}

fn bench_maven(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(MAVEN_XML);
    let xee_runner = XeeXPathRunner::new(MAVEN_XML);
    let xrust_runner = XrustRunner::new(MAVEN_XML);
    let amxml_runner = AmxmlRunner::new(MAVEN_XML);

    for (query_name, xpath) in MAVEN_QUERIES_TIER1 {
        let mut group = c.benchmark_group(format!("realworld/maven/{query_name}"));
        bench_one!(group, &sxd_runner, "sxd-xpath", xpath);
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }
}

fn bench_osm(c: &mut Criterion) {
    let sxd_runner = SxdXPathRunner::new(OSM_XML);
    let xee_runner = XeeXPathRunner::new(OSM_XML);
    let xrust_runner = XrustRunner::new(OSM_XML);
    let amxml_runner = AmxmlRunner::new(OSM_XML);

    for (query_name, xpath) in OSM_QUERIES_TIER1 {
        let mut group = c.benchmark_group(format!("realworld/osm/{query_name}"));
        bench_one!(group, &sxd_runner, "sxd-xpath", xpath);
        bench_one!(group, &xee_runner, "xee-xpath", xpath);
        bench_one!(group, &xrust_runner, "xrust", xpath);
        bench_one!(group, &amxml_runner, "amxml", xpath);
        group.finish();
    }
}

criterion_group!(benches, bench_rss, bench_maven, bench_osm);
criterion_main!(benches);
