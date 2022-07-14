use crate::error::{Error, StanzaParseError};
use jid::BareJid;
use libstrophe::Stanza;
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter, EnumString};

use super::namespace::Namespace;

#[derive(Debug, PartialEq, Display, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum MessageKind {
    /// The message is sent in the context of a one-to-one chat conversation.
    Chat,
    /// An error has occurred related to a previous message sent by the sender.
    Error,
    /// The message is sent in the context of a multi-user chat environment.
    Groupchat,
    /// The message is probably generated by an automated service that delivers or
    /// broadcasts content.
    Headline,
    /// The message is a single message that is sent outside the context of a
    /// one-to-one conversation or groupchat, and to which it is expected that the
    /// recipient will reply.
    Normal,
}

#[derive(Debug, PartialEq, EnumIter, Display)]
#[strum(serialize_all = "lowercase")]
pub enum ChatState {
    /// User is actively participating in the chat session.
    Active,
    /// User has not been actively participating in the chat session.
    Composing,
    /// User has effectively ended their participation in the chat session.
    Gone,
    /// User is composing a message.
    Inactive,
    /// User had been composing but now has stopped.
    Paused,
}

type MessageId = String;

#[derive(Debug, PartialEq)]
pub struct Message {
    pub from: BareJid,
    /// An instant messaging client SHOULD specify an intended recipient for a message by
    /// providing the JID of an entity other than the sender in the 'to' attribute of the
    /// <message/> stanza. If the message is being sent in reply to a message previously
    /// received from an address of the form <user@domain/resource> (e.g., within the
    /// context of a chat session), the value of the 'to' address SHOULD be of the form
    /// <user@domain/resource> rather than of the form <user@domain> unless the sender
    /// has knowledge (via presence) that the intended recipient's resource is no longer
    /// available. If the message is being sent outside the context of any existing chat
    /// session or received message, the value of the 'to' address SHOULD be of the form
    /// <user@domain> rather than of the form <user@domain/resource>.
    pub to: Option<BareJid>,

    pub id: Option<MessageId>,

    /// The 'type' attribute of a message stanza is RECOMMENDED; if included, it specifies
    /// the conversational context of the message, thus providing a hint regarding
    /// presentation (e.g., in a GUI).
    pub kind: Option<MessageKind>,

    /// The <body/> element contains human-readable XML character data that specifies the
    /// textual contents of the message; this child element is normally included but is
    /// OPTIONAL. The <body/> element MUST NOT possess any attributes, with the exception
    /// of the 'xml:lang' attribute. Multiple instances of the <body/> element MAY be
    /// included but only if each instance possesses an 'xml:lang' attribute with a
    /// distinct language value. The <body/> element MUST NOT contain mixed content.
    pub body: Option<String>,

    pub chat_state: Option<ChatState>,

    /// If set, the message the same id should be replaced.
    pub replace: Option<MessageId>,

    /// If the message stanza is of type "error", it MUST include an <error/> child.
    pub error: Option<String>,
}

impl Message {
    pub fn new_chat_message(
        from: BareJid,
        to: BareJid,
        id: Option<MessageId>,
        body: impl Into<String>,
        chat_state: Option<ChatState>,
    ) -> Self {
        Message {
            from,
            to: Some(to),
            id,
            kind: Some(MessageKind::Chat),
            body: Some(body.into()),
            chat_state,
            replace: None,
            error: None,
        }
    }
}

impl TryFrom<&Stanza> for Message {
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        Ok(Message {
            from: stanza
                .from()
                .ok_or(StanzaParseError::missing_attribute("from", stanza))
                .and_then(|str| BareJid::from_str(str).map_err(Into::into))?,
            to: stanza
                .get_attribute("to")
                .and_then(|s| BareJid::from_str(s).ok()),
            id: stanza.get_attribute("id").map(|s| s.to_string()),
            kind: stanza
                .get_attribute("type")
                .map(|s| s.to_string())
                .or_else(|| stanza.get_child_by_name("type").and_then(|n| n.text()))
                .and_then(|s| s.parse::<MessageKind>().ok()),
            body: stanza.get_child_by_name("body").and_then(|n| n.text()),
            chat_state: stanza.try_into().ok(),
            replace: stanza
                .get_child_by_name_and_ns("replace", Namespace::LastMessageCorrection)
                .and_then(|n| n.get_attribute("id").map(|s| s.to_string())),
            error: stanza.get_child_by_name("error").and_then(|n| n.text()),
        })
    }
}

impl TryFrom<&Stanza> for ChatState {
    type Error = ();

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        for state in ChatState::iter() {
            if stanza
                .get_child_by_name_and_ns(state.to_string(), Namespace::ChatStates)
                .is_some()
            {
                return Ok(state);
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use libstrophe::Stanza;

    use super::*;

    #[test]
    fn test_deserialize_empty_message() {
        let message = r#"
        <message from="valerian@prose.org"/>
        "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: None,
                id: None,
                kind: None,
                body: None,
                chat_state: None,
                replace: None,
                error: None,
            }
        );
    }

    #[test]
    fn test_deserialize_full_message() {
        let message = r#"
      <message from="valerian@prose.org/mobile" to="marc@prose.org/home" id="purplecf8f33c0" type="chat">
        <body>How is it going?</body>
        <active xmlns="http://jabber.org/protocol/chatstates"/>
      </message>
      "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                id: Some("purplecf8f33c0".to_string()),
                kind: Some(MessageKind::Chat),
                body: Some("How is it going?".to_string()),
                chat_state: Some(ChatState::Active),
                replace: None,
                error: None,
            }
        );
    }

    #[test]
    fn test_deserialize_correction_message() {
        let message = r#"
      <message from="valerian@prose.org/mobile" to="marc@prose.org/home" id="purplecf8f33c0" type="chat">
        <body>This is a correction</body>
        <replace id="message-id-1" xmlns="urn:xmpp:message-correct:0"/>
      </message>
      "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                id: Some("purplecf8f33c0".to_string()),
                kind: Some(MessageKind::Chat),
                body: Some("This is a correction".to_string()),
                chat_state: None,
                replace: Some("message-id-1".to_string()),
                error: None,
            }
        );
    }

    #[test]
    fn test_deserializes_chat_state_message() {
        let message = r#"
      <message from="valerian@prose.org/mobile" to="marc@prose.org/home" id="purplecf8f33c0" type="chat">
        <paused xmlns="http://jabber.org/protocol/chatstates"/>
      </message>
      "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: Some(BareJid::from_str("marc@prose.org").unwrap()),
                id: Some("purplecf8f33c0".to_string()),
                kind: Some(MessageKind::Chat),
                body: None,
                chat_state: Some(ChatState::Paused),
                replace: None,
                error: None,
            }
        );
    }

    #[test]
    fn test_ignores_chat_state_with_wrong_namespace() {
        let message = r#"
      <message from="valerian@prose.org/mobile" type="chat">
        <paused xmlns="something"/>
      </message>
      "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: None,
                id: None,
                kind: Some(MessageKind::Chat),
                body: None,
                chat_state: None,
                replace: None,
                error: None,
            }
        );
    }

    #[test]
    fn test_deserialize_error_message() {
        let message = r#"
    <message from="valerian@prose.org/mobile" type="error">
      <error>Something went wrong</error>
    </message>
    "#;

        let stanza = Stanza::from_str(message);
        let message = Message::try_from(&stanza).unwrap();

        assert_eq!(
            message,
            Message {
                from: BareJid::from_str("valerian@prose.org").unwrap(),
                to: None,
                id: None,
                kind: Some(MessageKind::Error),
                body: None,
                chat_state: None,
                replace: None,
                error: Some("Something went wrong".to_string()),
            }
        );
    }
}
