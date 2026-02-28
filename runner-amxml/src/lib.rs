use amxml::dom::{new_document, NodePtr};
use common::XPathRunner;

pub struct AmxmlRunner {
    doc: NodePtr,
}

impl XPathRunner for AmxmlRunner {
    fn new(xml: &str) -> Self {
        let doc = new_document(xml).expect("failed to parse XML");
        Self { doc }
    }

    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String> {
        let root = self.doc.root_element();
        let nodeset = root
            .get_nodeset(xpath)
            .map_err(|e| format!("XPath error: {e:?}"))?;
        Ok(nodeset.iter().map(|n| n.value()).collect())
    }
}
