// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::Jid;
use minidom::Element;
use strum_macros::{Display, EnumString};
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::{Body, MessageType, Subject};
use xmpp_parsers::message_correct::Replace;
use xmpp_parsers::stanza_error::StanzaError;

use prose_utils::id_string;

use crate::ns;
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::stanza_id::{OriginId, StanzaId};
use crate::stanza::message::{carbons, Fallback, Reactions};
use crate::stanza::message::{chat_marker, mam};
use crate::stanza::muc;

id_string!(Id);

// We're redeclaring ChatState here since this makes it easier to work with when saving it to the
// SQL db.
#[derive(
    Debug, PartialEq, Display, EnumString, Clone, serde::Serialize, serde::Deserialize, Default,
)]
#[strum(serialize_all = "lowercase")]
pub enum ChatState {
    /// User is actively participating in the chat session.
    Active,
    /// User has not been actively participating in the chat session.
    Composing,
    /// User has effectively ended their participation in the chat session.
    #[default]
    Gone,
    /// User is composing a message.
    Inactive,
    /// User had been composing but now has stopped.
    Paused,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Message {
    pub from: Option<Jid>,
    pub to: Option<Jid>,
    pub id: Option<Id>,
    pub stanza_id: Option<StanzaId>,
    pub origin_id: Option<OriginId>,
    pub r#type: MessageType,
    pub body: Option<String>,
    pub subject: Option<String>,
    pub chat_state: Option<ChatState>,
    pub replace: Option<Id>,
    pub reactions: Option<Reactions>,
    pub fastening: Option<ApplyTo>,
    pub fallback: Option<Fallback>,
    pub delay: Option<Delay>,
    pub markable: bool,
    pub displayed_marker: Option<chat_marker::Displayed>,
    pub received_marker: Option<chat_marker::Received>,
    pub acknowledged_marker: Option<chat_marker::Acknowledged>,
    pub archived_message: Option<mam::ArchivedMessage>,
    pub sent_carbon: Option<carbons::Sent>,
    pub received_carbon: Option<carbons::Received>,
    pub store: Option<bool>,
    pub direct_invite: Option<muc::DirectInvite>,
    pub mediated_invite: Option<muc::MediatedInvite>,
    pub error: Option<StanzaError>,
}

impl Message {
    pub fn new() -> Self {
        Message::default()
    }
}

impl TryFrom<xmpp_parsers::message::Message> for Message {
    type Error = anyhow::Error;

    fn try_from(root: xmpp_parsers::message::Message) -> Result<Self, Self::Error> {
        let mut message = Message::new();

        message.body = root
            .get_best_body(vec![])
            .map(|(_, body)| body.0.to_string());

        message.subject = root
            .get_best_subject(vec![])
            .map(|(_, subject)| subject.0.to_string());

        for payload in root.payloads.into_iter() {
            match payload {
                _ if payload.is("stanza-id", ns::SID) => {
                    message.stanza_id = Some(StanzaId::try_from(payload)?)
                }
                _ if payload.is("origin-id", ns::SID) => {
                    message.origin_id = Some(OriginId::try_from(payload)?)
                }
                _ if payload.has_ns(ns::CHATSTATES) => {
                    message.chat_state = Some(payload.name().parse()?)
                }
                _ if payload.is("replace", ns::MESSAGE_CORRECT) => {
                    message.replace = Some(Replace::try_from(payload)?.id.into())
                }
                _ if payload.is("reactions", ns::REACTIONS) => {
                    message.reactions = Some(Reactions::try_from(payload)?);
                }
                _ if payload.is("apply-to", ns::FASTEN) => {
                    message.fastening = Some(ApplyTo::try_from(payload)?)
                }
                _ if payload.is("fallback", ns::FALLBACK) => {
                    message.fallback = Some(Fallback::try_from(payload)?)
                }
                _ if payload.is("delay", ns::DELAY) => {
                    message.delay = Some(Delay::try_from(payload)?)
                }
                _ if payload.is("markable", ns::CHAT_MARKERS) => message.markable = true,
                _ if payload.is("received", ns::CHAT_MARKERS) => {
                    message.received_marker = Some(chat_marker::Received::try_from(payload)?)
                }
                _ if payload.is("displayed", ns::CHAT_MARKERS) => {
                    message.displayed_marker = Some(chat_marker::Displayed::try_from(payload)?)
                }
                _ if payload.is("acknowledged", ns::CHAT_MARKERS) => {
                    message.acknowledged_marker =
                        Some(chat_marker::Acknowledged::try_from(payload)?)
                }
                _ if payload.is("result", ns::MAM) => {
                    message.archived_message = Some(mam::ArchivedMessage::try_from(payload)?)
                }
                _ if payload.is("sent", ns::CARBONS) => {
                    message.sent_carbon = Some(carbons::Sent::try_from(payload)?)
                }
                _ if payload.is("received", ns::CARBONS) => {
                    message.received_carbon = Some(carbons::Received::try_from(payload)?)
                }
                _ if payload.is("x", ns::DIRECT_MUC_INVITATIONS) => {
                    message.direct_invite = Some(muc::DirectInvite::try_from(payload)?)
                }
                _ if payload.is("x", ns::MUC_USER) => {
                    message.mediated_invite = Some(muc::MediatedInvite::try_from(payload)?)
                }
                _ if payload.is("error", ns::DEFAULT_NS) => {
                    message.error = Some(StanzaError::try_from(payload)?)
                }
                _ => (),
            }
        }

        message.from = root.from;
        message.to = root.to;
        message.id = root.id.map(Into::into);
        message.r#type = root.type_;

        Ok(message)
    }
}

impl TryFrom<Element> for Message {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Message::try_from(xmpp_parsers::message::Message::try_from(value)?)
    }
}

impl From<Message> for Element {
    fn from(value: Message) -> Self {
        xmpp_parsers::message::Message::from(value).into()
    }
}

impl From<Message> for xmpp_parsers::message::Message {
    fn from(value: Message) -> Self {
        let mut message = xmpp_parsers::message::Message::new(None);
        message.to = value.to;
        message.from = value.from;
        message.id = value.id.map(|id| id.into_inner());
        message.type_ = value.r#type;

        if let Some(body) = value.body {
            message.bodies.insert("".into(), Body(body));
        }
        if let Some(subject) = value.subject {
            message.subjects.insert("".into(), Subject(subject));
        }
        if let Some(stanza_id) = value.stanza_id {
            message.payloads.push(stanza_id.into())
        }
        if let Some(origin_id) = value.origin_id {
            message.payloads.push(origin_id.into())
        }
        if let Some(chat_state) = value.chat_state {
            message
                .payloads
                .push(Element::builder(chat_state.to_string(), ns::CHATSTATES).build());
        }
        if let Some(replace) = value.replace {
            message.payloads.push(
                Replace {
                    id: replace.into_inner(),
                }
                .into(),
            );
        }
        if let Some(reactions) = value.reactions {
            message.payloads.push(reactions.into());
        }
        if let Some(fastening) = value.fastening {
            message.payloads.push(fastening.into());
        }
        if let Some(fallback) = value.fallback {
            message.payloads.push(fallback.into());
        }
        if let Some(delay) = value.delay {
            message.payloads.push(delay.into());
        }
        if value.markable {
            message.payloads.push(chat_marker::Markable {}.into());
        }
        if let Some(received) = value.received_marker {
            message.payloads.push(received.into());
        }
        if let Some(displayed) = value.displayed_marker {
            message.payloads.push(displayed.into());
        }
        if let Some(acknowledged) = value.acknowledged_marker {
            message.payloads.push(acknowledged.into());
        }
        if let Some(archived_message) = value.archived_message {
            message.payloads.push(archived_message.into());
        }
        if let Some(received_carbon) = value.received_carbon {
            message.payloads.push(received_carbon.into());
        }
        if let Some(sent_carbon) = value.sent_carbon {
            message.payloads.push(sent_carbon.into());
        }
        if let Some(store) = value.store {
            message.payloads.push(
                Element::builder(if store { "store" } else { "no-store" }, ns::HINTS).build(),
            );
        }
        if let Some(direct_invite) = value.direct_invite {
            message.payloads.push(direct_invite.into())
        }
        if let Some(mediated_invite) = value.mediated_invite {
            message.payloads.push(mediated_invite.into())
        }
        message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stanza::message::mam::ArchivedMessage;
    use crate::stanza::message::Forwarded;
    use crate::stanza::muc::{DirectInvite, Invite, MediatedInvite};
    use crate::{bare, jid};
    use anyhow::Result;
    use xmpp_parsers::mam::QueryId;
    use xmpp_parsers::message::Message as RawMessage;

    #[test]
    fn test_body() -> Result<()> {
        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_body("en".into(), "Hello World".into()),
        )?;
        assert_eq!(message.body, Some("Hello World".to_string()));
        Ok(())
    }

    #[test]
    fn test_subject() -> Result<()> {
        let mut raw = RawMessage::chat(jid!("recv@prose.org"));
        raw.subjects
            .insert("en".into(), Subject("Important Subject".to_string()));

        let message = Message::try_from(raw)?;
        assert_eq!(message.subject, Some("Important Subject".to_string()));
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

        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(invite.clone()),
        )?;
        assert_eq!(message.direct_invite, Some(invite));
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

        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(invite.clone()),
        )?;
        assert_eq!(message.mediated_invite, Some(invite));
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

        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(archived_message.clone()),
        )?;
        assert_eq!(message.archived_message, Some(archived_message));
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

        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(received_carbon.clone()),
        )?;
        assert_eq!(message.received_carbon, Some(received_carbon));
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

        let message = Message::try_from(
            RawMessage::chat(jid!("recv@prose.org")).with_payload(sent_carbon.clone()),
        )?;
        assert_eq!(message.sent_carbon, Some(sent_carbon));
        Ok(())
    }
}
