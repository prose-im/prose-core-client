// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::{Error, StanzaParseError};
use crate::types::namespace::Namespace;
use crate::MessageId;
use libstrophe::Stanza;

/// In order to mark that a payload applies to a previous message, a message is sent containing
/// an "apply-to" element in the namespace "urn:xmpp:fasten:0", with attribute "id" that contains
/// the Unique and Stable Stanza IDs (XEP-0359) [3] origin-id of the stanza to which it applies,
/// the children of which element are those that apply to the previous message (these are
/// "wrapped payloads" because they are wrapped inside the <apply-to> element). The id of this
/// apply-to-containing message is unimportant, and the type SHOULD be "normal".
#[derive(Debug, PartialEq)]
pub struct MessageFastening {
    /// The id of the message to which the payload should be fastened.
    pub id: MessageId,
    /// When true, the targeted message should be retracted.
    pub retract: bool,
}

impl MessageFastening {
    pub fn new(id: MessageId, retract: bool) -> Self {
        MessageFastening { id, retract }
    }
}

impl TryFrom<&Stanza> for MessageFastening {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(MessageFastening {
            id: stanza
                .id()
                .ok_or(StanzaParseError::missing_attribute("id", stanza))
                .map(|s| s.into())?,
            retract: stanza
                .get_child_by_name_and_ns("retract", Namespace::Retract)
                .is_some(),
        })
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_empty_fastening() {
        let fastening = r#"
        <apply-to xmlns="urn:xmpp:fasten:0" id="origin-id-1"/>
        "#;

        let stanza = Stanza::from_str(fastening);
        let fastening = MessageFastening::try_from(&stanza).unwrap();

        assert_eq!(
            fastening,
            MessageFastening {
                id: "origin-id-1".into(),
                retract: false
            }
        );
    }

    #[test]
    fn test_deserialize_retraction() {
        let fastening = r#"
        <apply-to xmlns="urn:xmpp:fasten:0" id="message-id">
          <retract xmlns='urn:xmpp:message-retract:0'/>
        </apply-to>
        "#;

        let stanza = Stanza::from_str(fastening);
        let fastening = MessageFastening::try_from(&stanza).unwrap();

        assert_eq!(
            fastening,
            MessageFastening {
                id: "message-id".into(),
                retract: true
            }
        );
    }
}
