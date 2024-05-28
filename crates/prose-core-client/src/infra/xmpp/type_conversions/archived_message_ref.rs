// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;

use prose_xmpp::{ElementExt, ParseError};

use crate::domain::messaging::models::ArchivedMessageRef;

pub mod ns {
    pub const PROSE_ARCHIVED_MESSAGE_REF: &str = "https://prose.org/protocol/archived_message_ref";
}

impl From<ArchivedMessageRef> for Element {
    fn from(value: ArchivedMessageRef) -> Self {
        Element::builder("archived-message-ref", ns::PROSE_ARCHIVED_MESSAGE_REF)
            .attr("stanza-id", value.stanza_id)
            .attr("ts", value.timestamp.to_rfc3339())
            .build()
    }
}

impl TryFrom<Element> for ArchivedMessageRef {
    type Error = ParseError;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Self {
            stanza_id: value.attr_req("stanza-id")?.into(),
            timestamp: value
                .attr_req("ts")?
                .parse()
                .map_err(|err: chrono::ParseError| ParseError::Generic {
                    msg: err.to_string(),
                })?,
        })
    }
}
