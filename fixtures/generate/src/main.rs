//! Generate artificial XML fixtures for the rust-xpath-benchmark suite.
//!
//! Run from the repository root:
//!
//!     cargo run -p generate
//!
//! Output files (written to the fixtures/ directory):
//!
//!   small.xml      ~10 books,     ~2 KB
//!   medium.xml     ~1 000 books,  ~160 KB
//!   large.xml      ~50 000 books, ~8 MB
//!   deep-nest.xml  200 levels of nesting with a single book at the leaf
//!   wide.xml       ~100 000 sibling book elements (flat)

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// Default random seeds
// ---------------------------------------------------------------------------

const DEFAULT_SEED_BOOKSTORE: u64 = 42;
const DEFAULT_SEED_WIDE: u64 = 123;

// ---------------------------------------------------------------------------
// Minimal LCG random number generator (no external dependencies)
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        // Knuth multiplicative LCG (64-bit)
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0
    }

    fn next_usize(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Uniform float in [lo, hi).
    fn next_f64(&mut self, lo: f64, hi: f64) -> f64 {
        let bits = self.next_u64() >> 11; // 53-bit mantissa
        let f = bits as f64 / (1u64 << 53) as f64; // [0, 1)
        lo + f * (hi - lo)
    }

    fn next_range(&mut self, lo: u32, hi: u32) -> u32 {
        lo + (self.next_u64() % (hi - lo + 1) as u64) as u32
    }

    fn choose<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        &slice[self.next_usize(slice.len())]
    }
}

// ---------------------------------------------------------------------------
// Shared vocabulary
// ---------------------------------------------------------------------------

const CATEGORIES: &[&str] = &[
    "fiction",
    "non-fiction",
    "science",
    "history",
    "biography",
    "technology",
];
const LANGS: &[&str] = &["en", "ja", "fr", "de", "es"];
const FIRST_NAMES: &[&str] = &[
    "Alice", "Bob", "Carol", "David", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
];
const LAST_NAMES: &[&str] = &[
    "Smith", "Jones", "Brown", "Taylor", "Wilson", "Davis", "Clark", "Lewis", "Hall", "Young",
];
const TITLE_WORDS: &[&str] = &[
    "The", "A", "An", "Great", "Lost", "Hidden", "Dark", "Bright", "Silent", "Deep", "World",
    "Time", "Life", "Mind", "Heart", "Soul", "Land", "Sea", "Sky", "Fire",
];

fn make_title(i: usize, rng: &mut Rng) -> String {
    format!(
        "{} {} {}",
        rng.choose(TITLE_WORDS),
        rng.choose(TITLE_WORDS),
        i
    )
}

fn make_author(rng: &mut Rng) -> String {
    format!("{} {}", rng.choose(FIRST_NAMES), rng.choose(LAST_NAMES))
}

// ---------------------------------------------------------------------------
// small.xml / medium.xml / large.xml  (bookstore with N books)
// ---------------------------------------------------------------------------

fn make_bookstore(n: usize, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let mut out = String::with_capacity(n * 160);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<bookstore>\n");
    for i in 1..=n {
        let cat = rng.choose(CATEGORIES);
        let lang = rng.choose(LANGS);
        let year = rng.next_range(1900, 2024);
        let price = (rng.next_f64(5.0, 50.0) * 100.0).round() / 100.0;
        let title = make_title(i, &mut rng);
        let author = make_author(&mut rng);
        writeln!(out, "  <book category=\"{cat}\">").unwrap();
        writeln!(out, "    <title lang=\"{lang}\">{title}</title>").unwrap();
        writeln!(out, "    <author>{author}</author>").unwrap();
        writeln!(out, "    <year>{year}</year>").unwrap();
        writeln!(out, "    <price>{price:.2}</price>").unwrap();
        out.push_str("  </book>\n");
    }
    out.push_str("</bookstore>");
    out
}

// ---------------------------------------------------------------------------
// deep-nest.xml  (depth levels of nesting, no random content)
// ---------------------------------------------------------------------------

fn make_deep_nest(depth: usize) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    for i in 1..=depth {
        let indent = "  ".repeat(i - 1);
        writeln!(out, "{indent}<level n=\"{i}\">").unwrap();
    }
    let indent = "  ".repeat(depth);
    writeln!(out, "{indent}<book category=\"fiction\">").unwrap();
    writeln!(out, "{indent}  <title lang=\"en\">The Deepest Book</title>").unwrap();
    writeln!(out, "{indent}  <author>Deep Author</author>").unwrap();
    writeln!(out, "{indent}  <year>2024</year>").unwrap();
    writeln!(out, "{indent}  <price>9.99</price>").unwrap();
    writeln!(out, "{indent}</book>").unwrap();
    for i in (1..=depth).rev() {
        let indent = "  ".repeat(i - 1);
        writeln!(out, "{indent}</level>").unwrap();
    }
    // trim trailing newline
    if out.ends_with('\n') {
        out.pop();
    }
    out
}

// ---------------------------------------------------------------------------
// wide.xml  (n sibling book elements, flat)
// ---------------------------------------------------------------------------

fn make_wide(n: usize, seed: u64) -> String {
    let mut rng = Rng::new(seed);
    let mut out = String::with_capacity(n * 100);
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<bookstore>\n");
    for i in 1..=n {
        let cat = rng.choose(CATEGORIES);
        let lang = rng.choose(LANGS);
        let price = (rng.next_f64(5.0, 50.0) * 100.0).round() / 100.0;
        writeln!(
            out,
            "  <book id=\"{i}\" category=\"{cat}\"><title lang=\"{lang}\">Book {i}</title><price>{price:.2}</price></book>"
        )
        .unwrap();
    }
    out.push_str("</bookstore>");
    out
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn write_fixture(dir: &Path, name: &str, content: &str) {
    let path = dir.join(name);
    fs::write(&path, content).unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    println!("  wrote {name:<20}  {size:>12} bytes");
}

fn main() {
    // fixtures/ is the parent directory of this crate's Cargo.toml.
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("fixtures/generate/ has no parent");

    println!("Generating artificial XML fixtures …");

    write_fixture(
        fixtures_dir,
        "small.xml",
        &make_bookstore(10, DEFAULT_SEED_BOOKSTORE),
    );
    write_fixture(
        fixtures_dir,
        "medium.xml",
        &make_bookstore(1_000, DEFAULT_SEED_BOOKSTORE),
    );
    write_fixture(
        fixtures_dir,
        "large.xml",
        &make_bookstore(50_000, DEFAULT_SEED_BOOKSTORE),
    );
    write_fixture(fixtures_dir, "deep-nest.xml", &make_deep_nest(200));
    write_fixture(
        fixtures_dir,
        "wide.xml",
        &make_wide(100_000, DEFAULT_SEED_WIDE),
    );

    println!("Done.");
}
