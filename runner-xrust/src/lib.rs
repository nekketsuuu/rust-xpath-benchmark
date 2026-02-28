use common::XPathRunner;
use xrust::item::{Item, Node};
use xrust::parser::xml::parse as xml_parse;
use xrust::parser::xpath::parse as xpath_parse;
use xrust::parser::ParseError;
use xrust::transform::context::{ContextBuilder, StaticContextBuilder};
use xrust::trees::smite::RNode;
use xrust::xdmerror::{Error, ErrorKind};

pub struct XrustRunner {
    doc: RNode,
}

impl XPathRunner for XrustRunner {
    fn new(xml: &str) -> Self {
        let doc = RNode::new_document();
        xml_parse(
            doc.clone(),
            xml,
            Some(|_: &_| Err(ParseError::MissingNameSpace)),
        )
        .expect("failed to parse XML");
        Self { doc }
    }

    fn evaluate(&self, xpath: &str) -> Result<Vec<String>, String> {
        let transform = xpath_parse::<RNode>(xpath, None, None)
            .map_err(|e| format!("XPath parse error: {e:?}"))?;

        let mut static_context = StaticContextBuilder::new()
            .message(|_| Ok(()))
            .fetcher(|_| Ok(String::new()))
            .parser(|_| {
                Err(Error::new(
                    ErrorKind::NotImplemented,
                    "external document fetching not supported in benchmark",
                ))
            })
            .build();

        let context = ContextBuilder::new()
            .context(vec![Item::Node(self.doc.clone())])
            .build();

        let sequence = context
            .dispatch(&mut static_context, &transform)
            .map_err(|e| format!("XPath evaluation error: {e:?}"))?;

        Ok(sequence.iter().map(|item| item.to_string()).collect())
    }
}
