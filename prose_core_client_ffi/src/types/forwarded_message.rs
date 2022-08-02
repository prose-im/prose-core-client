use crate::error::{Error, StanzaParseError};
use crate::types::delay::Delay;
use crate::types::message::Message;
use crate::types::namespace::Namespace;
use libstrophe::Stanza;
use std::ops::Deref;

// https://xmpp.org/extensions/xep-0297.html

#[derive(Debug, PartialEq)]
pub struct ForwardedMessage {
    pub delay: Option<Delay>,
    pub message: Message,
}

impl ForwardedMessage {
    pub fn new(message: Message, delay: Option<Delay>) -> Self {
        ForwardedMessage { message, delay }
    }
}

impl TryFrom<&Stanza> for ForwardedMessage {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(ForwardedMessage::new(
            stanza
                .get_child_by_name("message")
                .ok_or(Error::StanzaParseError {
                    error: StanzaParseError::missing_child_node("message", stanza),
                })
                .and_then(|n| Message::try_from(n.deref()))?,
            stanza
                .get_child_by_name_and_ns("delay", Namespace::Delay)
                .and_then(|n| Delay::try_from(n.deref()).ok()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::message::MessageKind;
    use jid::BareJid;
    use libstrophe::Stanza;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_message() {
        let message = r#"
        <forwarded xmlns="urn:xmpp:forward:0">
          <delay xmlns="urn:xmpp:delay" stamp="2022-07-07T08:35:28Z"/>
          <message id="purple29a25424" xmlns="jabber:client" to="marc@prose.org" type="chat" from="cram@prose.org/mb">
            <body>yo!</body>
          </message>
        </forwarded>
        "#;

        let stanza = Stanza::from_str(message);
        let message = ForwardedMessage::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            ForwardedMessage::new(
                Message {
                    from: BareJid::from_str("cram@prose.org/mb").unwrap(),
                    to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                    id: Some("purple29a25424".into()),
                    kind: Some(MessageKind::Chat),
                    body: Some("yo!".to_string()),
                    chat_state: None,
                    replace: None,
                    reactions: None,
                    error: None,
                },
                Some(Delay::new(1657182928, None))
            )
        );
    }

    #[test]
    fn test_deserialize_message_without_delay() {
        let message = r#"
        <forwarded xmlns="urn:xmpp:forward:0">
          <message id="purple29a25424" xmlns="jabber:client" to="marc@prose.org" type="chat" from="cram@prose.org/mb">
            <body>yo!</body>
          </message>
        </forwarded>
        "#;

        let stanza = Stanza::from_str(message);
        let message = ForwardedMessage::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            ForwardedMessage::new(
                Message {
                    from: BareJid::from_str("cram@prose.org/mb").unwrap(),
                    to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                    id: Some("purple29a25424".into()),
                    kind: Some(MessageKind::Chat),
                    body: Some("yo!".to_string()),
                    chat_state: None,
                    replace: None,
                    reactions: None,
                    error: None,
                },
                None
            )
        );
    }
}
