// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::{Deref, DerefMut};

use anyhow::Result;
use minidom::Element;
use tracing::error;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::{Message as RawMessage, MessagePayload};
use xmpp_parsers::message_correct::Replace;
use xmpp_parsers::stanza_error::StanzaError;

use prose_utils::id_string;

use crate::ns;
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::muc_user::MucUser;
use crate::stanza::message::stanza_id::StanzaId;
use crate::stanza::message::{carbons, Reactions};
use crate::stanza::message::{chat_marker, mam};
use crate::stanza::muc;

id_string!(Id);

#[derive(Debug, PartialEq, Clone)]
pub struct Message(RawMessage);

impl Default for Message {
    fn default() -> Self {
        Self(RawMessage {
            from: None,
            to: None,
            id: None,
            type_: Default::default(),
            bodies: Default::default(),
            subjects: Default::default(),
            thread: None,
            payloads: vec![],
        })
    }
}

impl Message {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Deref for Message {
    type Target = RawMessage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Message {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Message {
    pub fn body(&self) -> Option<&str> {
        self.get_best_body(vec![])
            .as_ref()
            .map(|(_, body)| body.0.as_str())
    }

    pub fn subject(&self) -> Option<&str> {
        self.get_best_subject(vec![])
            .as_ref()
            .map(|(_, subject)| subject.0.as_str())
    }

    pub fn direct_invite(&self) -> Option<muc::DirectInvite> {
        self.typed_payload("x", ns::DIRECT_MUC_INVITATIONS)
    }

    pub fn mediated_invite(&self) -> Option<muc::MediatedInvite> {
        self.typed_payload("x", ns::MUC_USER)
    }

    pub fn archived_message(&self) -> Option<mam::ArchivedMessage> {
        self.typed_payload("result", ns::MAM)
    }

    pub fn received_carbon(&self) -> Option<carbons::Received> {
        self.typed_payload("received", ns::CARBONS)
    }

    pub fn sent_carbon(&self) -> Option<carbons::Sent> {
        self.typed_payload("sent", ns::CARBONS)
    }

    pub fn is_mam_message(&self) -> bool {
        self.payloads
            .iter()
            .find(|p| p.is("result", ns::MAM))
            .is_some()
    }

    pub fn chat_state(&self) -> Option<ChatState> {
        self.typed_payload_with_predicate(|p| p.has_ns(ns::CHATSTATES))
    }

    pub fn stanza_id(&self) -> Option<StanzaId> {
        self.typed_payload("stanza-id", ns::SID)
    }

    pub fn delay(&self) -> Option<Delay> {
        self.typed_payload("delay", ns::DELAY)
    }

    pub fn error(&self) -> Option<StanzaError> {
        self.typed_payload("error", ns::DEFAULT_NS)
    }

    pub fn reactions(&self) -> Option<Reactions> {
        self.typed_payload("reactions", ns::REACTIONS)
    }

    pub fn fastening(&self) -> Option<ApplyTo> {
        self.typed_payload("apply-to", ns::FASTEN)
    }

    pub fn replace(&self) -> Option<Id> {
        self.typed_payload::<Replace>("replace", ns::MESSAGE_CORRECT)
            .map(|r| r.id.into())
    }

    pub fn received_marker(&self) -> Option<chat_marker::Received> {
        self.typed_payload("received", ns::CHAT_MARKERS)
    }

    pub fn displayed_marker(&self) -> Option<chat_marker::Displayed> {
        self.typed_payload("displayed", ns::CHAT_MARKERS)
    }

    pub fn muc_user(&self) -> Option<MucUser> {
        self.typed_payload("x", ns::MUC_USER)
    }
}

impl Message {
    fn typed_payload<P: MessagePayload>(&self, name: &str, ns: &str) -> Option<P> {
        self.typed_payload_with_predicate(|p| p.is(name, ns))
    }

    fn typed_payload_with_predicate<P: MessagePayload, F>(&self, predicate: F) -> Option<P>
    where
        F: FnMut(&Element) -> bool,
    {
        let mut predicate = predicate;
        let Some(payload) = self.payloads.iter().find(|p| predicate(*p)) else {
            return None;
        };
        let Ok(payload) = P::try_from(payload.clone()) else {
            error!("Failed to parse message payload {}.", String::from(payload));
            return None;
        };
        Some(payload)
    }
}

impl From<Message> for Element {
    fn from(value: Message) -> Self {
        value.0.into()
    }
}

impl TryFrom<Element> for Message {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Message(xmpp_parsers::message::Message::try_from(value)?))
    }
}

impl From<Message> for RawMessage {
    fn from(value: Message) -> Self {
        value.0
    }
}

impl From<RawMessage> for Message {
    fn from(value: RawMessage) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use xmpp_parsers::mam::QueryId;
    use xmpp_parsers::message::{Message as RawMessage, Subject};

    use crate::stanza::message::mam::ArchivedMessage;
    use crate::stanza::message::Forwarded;
    use crate::stanza::muc::{DirectInvite, Invite, MediatedInvite};
    use crate::{bare, jid};

    use super::*;

    #[test]
    fn test_body() -> Result<()> {
        let message = Message::from(
            RawMessage::chat(jid!("recv@prose.org")).with_body("en".into(), "Hello World".into()),
        );
        assert_eq!(message.body(), Some("Hello World"));
        Ok(())
    }

    #[test]
    fn test_subject() -> Result<()> {
        let mut raw = RawMessage::chat(jid!("recv@prose.org"));
        raw.subjects
            .insert("en".into(), Subject("Important Subject".to_string()));

        let message = Message::from(raw);
        assert_eq!(message.subject(), Some("Important Subject"));
        Ok(())
    }

    #[test]
    fn test_direct_invite() -> Result<()> {
        let invite = DirectInvite {
            jid: bare!("user@prose.org"),
            password: Some("topsecret".to_string()),
            reason: Some("Who knows".to_string()),
            r#continue: None,
            thread: None,
        };

        let message =
            Message::from(RawMessage::chat(jid!("recv@prose.org")).with_payload(invite.clone()));
        assert_eq!(message.direct_invite(), Some(invite));
        Ok(())
    }

    #[test]
    fn test_mediated_invite() -> Result<()> {
        let invite = MediatedInvite {
            invites: vec![Invite {
                from: Some(jid!("sender@prose.org")),
                to: Some(jid!("recv@prose.org")),
                reason: Some("Some reason".to_string()),
            }],
            password: None,
        };

        let message =
            Message::from(RawMessage::chat(jid!("recv@prose.org")).with_payload(invite.clone()));
        assert_eq!(message.mediated_invite(), Some(invite));
        Ok(())
    }

    #[test]
    fn test_archived_message() -> Result<()> {
        let archived_message = ArchivedMessage {
            id: "message-id".into(),
            query_id: Some(QueryId("query-id".to_string())),
            forwarded: Forwarded {
                delay: None,
                stanza: None,
            },
        };

        let message = Message::from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(archived_message.clone()),
        );
        assert_eq!(message.archived_message(), Some(archived_message));
        Ok(())
    }

    #[test]
    fn test_received_carbon() -> Result<()> {
        let received_carbon = carbons::Received {
            forwarded: Forwarded {
                delay: None,
                stanza: Some(Box::new(Message::new().set_id("id-100".into()))),
            },
        };

        let message = Message::from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(received_carbon.clone()),
        );
        assert_eq!(message.received_carbon(), Some(received_carbon));
        Ok(())
    }

    #[test]
    fn test_sent_carbon() -> Result<()> {
        let sent_carbon = carbons::Sent {
            forwarded: Forwarded {
                delay: None,
                stanza: Some(Box::new(Message::new().set_id("id-100".into()))),
            },
        };

        let message = Message::from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(sent_carbon.clone()),
        );
        assert_eq!(message.sent_carbon(), Some(sent_carbon));
        Ok(())
    }
}
