use std::cell::RefCell;

use common::XPathRunner;
use xee_xpath::{DocumentHandle, Documents, Queries, Query, Sequence};

pub struct XeeXPathRunner {
    // Documents is not Clone, so we wrap it in RefCell to allow interior
    // mutability while keeping &self in the XPathRunner trait.
    documents: RefCell<Documents>,
    doc_handle: DocumentHandle,
}

impl XPathRunner for XeeXPathRunner {
    fn new(xml: &str) -> Self {
        let mut documents = Documents::new();
        let doc_handle = documents
            .add_string("http://benchmark.local/doc".try_into().unwrap(), xml)
            .expect("failed to parse XML");
        Self {
            documents: RefCell::new(documents),
            doc_handle,
        }
    }

    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String> {
        let queries = Queries::default();
        let q: xee_xpath::query::SequenceQuery = queries
            .sequence(xpath)
            .map_err(|e| format!("XPath compile error: {e}"))?;

        let mut documents = self.documents.borrow_mut();
        let sequence: Sequence = q
            .execute(&mut *documents, self.doc_handle)
            .map_err(|e| format!("XPath evaluation error: {e}"))?;

        // Convert each item to a string representation.
        // For atomic values: Atomic::to_string() returns Result<String, ErrorValue>.
        // For nodes: Xot::string_value() returns String directly.
        let xot = documents.xot();
        let results: Vec<String> = sequence
            .iter()
            .map(|item| match item {
                xee_xpath::Item::Atomic(a) => a.to_string().map_err(|e| format!("{e:?}")),
                xee_xpath::Item::Node(n) => Ok(xot.string_value(n)),
                xee_xpath::Item::Function(_) => Ok("<function>".to_string()),
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("XPath result conversion error: {e}"))?;
        Ok(results)
    }
}
