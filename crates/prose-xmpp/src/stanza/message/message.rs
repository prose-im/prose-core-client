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

use crate::ns;
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::stanza_id::{OriginId, StanzaId};
use crate::stanza::message::{carbons, Fallback, Reactions};
use crate::stanza::message::{chat_marker, mam};
use crate::stanza::muc;
use crate::util::id_string_macro::id_string;

id_string!(Id);

// We're redeclaring ChatState here since this makes it easier to work with when saving it to the
// SQL db.
#[derive(Debug, PartialEq, Display, EnumString, Clone, serde::Serialize, serde::Deserialize)]
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

// TODO: Fix tests

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_read_chat_state() -> Result<()> {
//         let message = r#"
//       <message from="valerian@prose.org/mobile" to="marc@prose.org/home" id="purplecf8f33c0" type="chat">
//         <body>How is it going?</body>
//         <active xmlns="http://jabber.org/protocol/chatstates"/>
//       </message>
//       "#;
//
//         let stanza = Message::from_str(message).unwrap();
//
//         // assert_eq!(stanza.chat_state(), Some(ChatState::Active));
//         assert_eq!(stanza.body().as_deref(), Some("How is it going?"));
//
//         Ok(())
//     }
//
//     #[test]
//     fn test_get_message_reactions() {
//         let message = r#"
//       <message from="a@prose.org" to="b@prose.org" id="id1" type="chat">
//         <reactions id="id2" xmlns='urn:xmpp:reactions:0'>
//             <reaction>👋</reaction>
//             <reaction>🐢</reaction>
//         </reactions>
//       </message>
//       "#;
//
//         let stanza = Message::from_str(message).unwrap();
//         assert_eq!(
//             stanza.message_reactions(),
//             Some(("id2".into(), vec!["👋".into(), "🐢".into()]))
//         );
//     }
//
//     #[test]
//     fn test_set_message_reactions() {
//         let stanza =
//             Message::new().set_message_reactions("id2".into(), vec!["👋".into(), "🐢".into()]);
//         let message = r#"<message><reactions id="id2" xmlns="urn:xmpp:reactions:0"><reaction>👋</reaction><reaction>🐢</reaction></reactions></message>"#;
//         assert_eq!(stanza.to_string(), message);
//     }
// }
