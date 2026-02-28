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

        // Convert each item to its XPath string value.
        // Item::string_value() handles all cases correctly:
        //   - Atomic values: canonical lexical representation (integers, doubles, etc.)
        //   - Nodes: string value via Xot::string_value()
        //   - Functions: returns FOTY0014 error
        let xot = documents.xot();
        let results: Vec<String> = sequence
            .iter()
            .map(|item| {
                item.string_value(xot)
                    .map_err(|e| format!("XPath result conversion error: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }
}
