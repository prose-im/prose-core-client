use crate::error::{Error, StanzaParseError};
use crate::MessageId;
use libstrophe::Stanza;

/// In order to mark that a payload applies to a previous message, a message is sent containing
/// an "apply-to" element in the namespace "urn:xmpp:fasten:0", with attribute "id" that contains
/// the Unique and Stable Stanza IDs (XEP-0359) [3] origin-id of the stanza to which it applies,
/// the children of which element are those that apply to the previous message (these are
/// "wrapped payloads" because they are wrapped inside the <apply-to> element). The id of this
/// apply-to-containing message is unimportant, and the type SHOULD be "normal".
#[derive(Debug, PartialEq)]
pub struct MessageReactions {
    /// The id of the message to which the payload should be fastened.
    pub id: MessageId,
    /// When true, the targeted message should be retracted.
    pub reactions: Vec<String>,
}

impl MessageReactions {
    pub fn new(id: MessageId, reactions: Vec<String>) -> Self {
        MessageReactions { id, reactions }
    }
}

impl TryFrom<&Stanza> for MessageReactions {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(MessageReactions {
            id: stanza
                .id()
                .ok_or(StanzaParseError::missing_attribute("id", stanza))
                .map(|s| s.into())?,
            reactions: stanza
                .children()
                .filter_map(|c| {
                    if c.name() != Some("reaction") {
                        return None;
                    }
                    return c.get_first_child().and_then(|c| c.text());
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_reactions() {
        let fastening = r#"
        <reactions id="381c37e0-8e05-42c0-beb6-fdb5fa6263ec" xmlns="urn:xmpp:reactions:0">
          <reaction>🥲</reaction>
          <reaction>:-)</reaction>
          <some_other_node>Should be ignored</some_other_node>
        </reactions>
        "#;

        let stanza = Stanza::from_str(fastening);
        let fastening = MessageReactions::try_from(&stanza).unwrap();

        assert_eq!(
            fastening,
            MessageReactions {
                id: "381c37e0-8e05-42c0-beb6-fdb5fa6263ec".into(),
                reactions: vec!["🥲", ":-)"].iter().map(|s| s.to_string()).collect()
            }
        );
    }
}
