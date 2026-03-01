//! Probe binary for timeout detection.
//!
//! Parses an XML fixture and runs a single XPath evaluation.  The parent
//! process spawns this with a wall-clock timeout and kills it if it takes
//! too long.  This avoids blocking the benchmark binary on slow cases.
//!
//! Usage:
//!     probe <library> <fixture> <xpath>
//!
//! Exit codes:
//!     0  — evaluate() succeeded
//!     1  — evaluate() returned Err (likely a SKIP list omission)
//!     2  — unknown library or fixture name

use common::XPathRunner;
use runner_amxml::AmxmlRunner;
use runner_sxd_xpath::SxdXPathRunner;
use runner_xee_xpath::XeeXPathRunner;
use runner_xrust::XrustRunner;

// Embed all fixture XML at compile time so the probe binary is self-contained.
const SMALL_XML: &str = include_str!("../../../fixtures/small.xml");
const MEDIUM_XML: &str = include_str!("../../../fixtures/medium.xml");
const LARGE_XML: &str = include_str!("../../../fixtures/large.xml");
const DEEP_XML: &str = include_str!("../../../fixtures/deep-nest.xml");
const WIDE_XML: &str = include_str!("../../../fixtures/wide.xml");
const RSS_XML: &str = include_str!("../../../fixtures/rss2-sample.xml");
const MAVEN_XML: &str = include_str!("../../../fixtures/maven-pom.xml");
const OSM_XML: &str = include_str!("../../../fixtures/osm-map.xml");

fn fixture_xml(name: &str) -> Option<&'static str> {
    match name {
        "small" => Some(SMALL_XML),
        "medium" => Some(MEDIUM_XML),
        "large" => Some(LARGE_XML),
        "deep-nest" => Some(DEEP_XML),
        "wide" => Some(WIDE_XML),
        "rss2-sample" => Some(RSS_XML),
        "maven-pom" => Some(MAVEN_XML),
        "osm-map" => Some(OSM_XML),
        _ => None,
    }
}

/// Run evaluate() with the given library and return the result.
fn run_evaluate(library: &str, xml: &str, xpath: &str) -> Result<Vec<String>, String> {
    match library {
        "sxd-xpath" => {
            let runner = SxdXPathRunner::new(xml);
            runner.evaluate(xpath)
        }
        "xee-xpath" => {
            let runner = XeeXPathRunner::new(xml);
            runner.evaluate(xpath)
        }
        "xrust" => {
            let runner = XrustRunner::new(xml);
            runner.evaluate(xpath)
        }
        "amxml" => {
            let runner = AmxmlRunner::new(xml);
            runner.evaluate(xpath)
        }
        _ => Err(format!("unknown library: {library}")),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: probe <library> <fixture> <xpath>");
        std::process::exit(2);
    }

    let library = &args[1];
    let fixture = &args[2];
    let xpath = &args[3];

    let Some(xml) = fixture_xml(fixture) else {
        eprintln!("Unknown fixture: {fixture}");
        std::process::exit(2);
    };

    match run_evaluate(library, xml, xpath) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("evaluate() error: {e}");
            std::process::exit(1);
        }
    }
}
