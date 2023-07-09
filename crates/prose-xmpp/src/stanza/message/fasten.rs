use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use crate::ns;
use crate::stanza::message;
use crate::util::ElementExt;

#[derive(Debug, PartialEq, Clone)]
pub struct ApplyTo {
    pub id: message::Id,
    pub clear: bool,
    pub payloads: Vec<Element>,
}

impl ApplyTo {
    pub fn new(id: message::Id) -> Self {
        ApplyTo {
            id,
            clear: false,
            payloads: vec![],
        }
    }

    pub fn with_payload<P: ApplyToPayload>(mut self, payload: P) -> Self {
        self.payloads.push(payload.into());
        self
    }

    pub fn with_payloads<P: ApplyToPayload>(
        mut self,
        payloads: impl IntoIterator<Item = P>,
    ) -> Self {
        self.payloads = payloads.into_iter().map(Into::into).collect();
        self
    }
}

impl Into<Element> for ApplyTo {
    fn into(self) -> Element {
        Element::builder("apply-to", ns::FASTEN)
            .attr("id", self.id)
            .attr("clear", if self.clear { Some("true") } else { None })
            .append_all(self.payloads)
            .build()
    }
}

impl TryFrom<Element> for ApplyTo {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("apply-to", ns::FASTEN)?;

        Ok(ApplyTo {
            id: value.req_attr("id")?.into(),
            clear: value
                .attr("clear")
                .map(|value| value.to_lowercase() == "true")
                .unwrap_or(false),
            payloads: value.children().map(|e| e.clone()).collect(),
        })
    }
}

impl MessagePayload for ApplyTo {}

pub trait ApplyToPayload: TryFrom<Element> + Into<Element> {}
