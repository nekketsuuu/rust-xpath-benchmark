use common::XPathRunner;
use sxd_document::parser as xml_parser;
use sxd_document::Package;
use sxd_xpath::{Context, Factory, Value};

pub struct SxdXPathRunner {
    package: Package,
}

impl XPathRunner for SxdXPathRunner {
    fn new(xml: &str) -> Self {
        let package = xml_parser::parse(xml).expect("failed to parse XML");
        Self { package }
    }

    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String> {
        let document = self.package.as_document();
        let factory = Factory::new();
        let compiled = factory
            .build(xpath)
            .map_err(|e| format!("XPath compile error: {e:?}"))?
            .ok_or_else(|| "no XPath compiled".to_string())?;
        let context = Context::new();
        let value = compiled
            .evaluate(&context, document.root())
            .map_err(|e| format!("XPath evaluation error: {e:?}"))?;

        let results = match value {
            Value::Nodeset(nodeset) => nodeset
                .document_order()
                .iter()
                .map(|n| n.string_value())
                .collect(),
            Value::String(s) => vec![s],
            Value::Number(n) => vec![n.to_string()],
            Value::Boolean(b) => vec![b.to_string()],
        };
        Ok(results)
    }
}
