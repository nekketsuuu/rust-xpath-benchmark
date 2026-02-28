/// A common interface for XPath benchmark runners.
///
/// Each runner wraps a specific XPath library. XML parsing is done in `new()`
/// and is intentionally excluded from benchmark measurements. Only `evaluate()`
/// is measured.
pub trait XPathRunner {
    /// Parse the given XML string and prepare the runner for XPath evaluation.
    /// This is called during benchmark setup and is **not** included in timing.
    fn new(xml: &str) -> Self;

    /// Evaluate the given XPath expression against the parsed document.
    ///
    /// Returns a list of string representations of the matched nodes or values.
    /// Returns `Err` if the expression is unsupported or evaluation fails.
    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String>;
}
