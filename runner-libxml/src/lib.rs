use common::XPathRunner;
use libxml::parser::Parser;
use libxml::tree::Document;
use libxml::xpath::Context;

// XPath result type constants from libxml2 (bindgen-generated).
// xmlXPathObjectType is c_uint (u32).
const XPATH_NODESET: u32 = 1;
const XPATH_BOOLEAN: u32 = 2;
const XPATH_NUMBER: u32 = 3;
const XPATH_STRING: u32 = 4;

pub struct LibxmlRunner {
    doc: Document,
}

impl XPathRunner for LibxmlRunner {
    fn new(xml: &str) -> Self {
        let parser = Parser::default();
        let doc = parser.parse_string(xml).expect("failed to parse XML");
        Self { doc }
    }

    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String> {
        let ctx =
            Context::new(&self.doc).map_err(|()| "failed to create XPath context".to_string())?;
        let obj = ctx
            .evaluate(xpath)
            .map_err(|()| format!("XPath evaluation failed: {xpath}"))?;

        // Inspect the underlying libxml2 result type via the public raw pointer.
        let result_type = unsafe { (*obj.ptr).type_ };

        match result_type {
            XPATH_NODESET => Ok(obj.get_nodes_as_str()),
            XPATH_BOOLEAN | XPATH_NUMBER | XPATH_STRING => Ok(vec![obj.to_string()]),
            _ => Err(format!("unsupported XPath result type: {result_type}")),
        }
    }
}
