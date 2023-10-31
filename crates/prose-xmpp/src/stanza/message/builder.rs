// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::Jid;

use minidom::Element;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::{Body, MessagePayload, MessageType, Subject};
use xmpp_parsers::message_correct::Replace;

use crate::ns;
use crate::stanza::message::chat_marker::{Acknowledged, Displayed, Received};
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::mam::ArchivedMessage;
use crate::stanza::message::message::Message;
use crate::stanza::message::{carbons, chat_marker, Fallback, Id, Reactions};

impl Message {
    pub fn set_to(mut self, to: impl Into<Jid>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub fn set_from(mut self, from: impl Into<Jid>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn set_id(mut self, id: Id) -> Self {
        self.id = Some(id.into_inner());
        self
    }

    pub fn set_type(mut self, r#type: MessageType) -> Self {
        self.type_ = r#type;
        self
    }

    pub fn set_body(mut self, body: impl Into<String>) -> Self {
        self.bodies.insert("".into(), Body(body.into()));
        self
    }

    pub fn set_subject(mut self, subject: impl Into<String>) -> Self {
        self.subjects.insert("".into(), Subject(subject.into()));
        self
    }

    pub fn add_payload<P: MessagePayload>(mut self, payload: P) -> Self {
        self.payloads.push(payload.into());
        self
    }

    pub fn set_chat_state(mut self, state: Option<ChatState>) -> Self {
        if let Some(state) = state {
            self.payloads.push(state.into());
        }
        self
    }

    pub fn set_replace(mut self, id: Id) -> Self {
        self.payloads.push(
            Replace {
                id: id.into_inner(),
            }
            .into(),
        );
        self
    }

    pub fn set_message_reactions(mut self, reactions: Reactions) -> Self {
        self.payloads.push(reactions.into());
        self
    }

    pub fn set_fastening(mut self, fastening: ApplyTo) -> Self {
        self.payloads.push(fastening.into());
        self
    }

    pub fn set_fallback(mut self, fallback: Fallback) -> Self {
        self.payloads.push(fallback.into());
        self
    }

    pub fn set_markable(mut self) -> Self {
        self.payloads.push(chat_marker::Markable {}.into());
        self
    }

    pub fn set_received_marker(mut self, marker: Received) -> Self {
        self.payloads.push(marker.into());
        self
    }

    pub fn set_displayed_marker(mut self, marker: Displayed) -> Self {
        self.payloads.push(marker.into());
        self
    }

    pub fn set_acknowledged_marker(mut self, marker: Acknowledged) -> Self {
        self.payloads.push(marker.into());
        self
    }

    pub fn set_archived_message(mut self, message: ArchivedMessage) -> Self {
        self.payloads.push(message.into());
        self
    }

    pub fn set_received_carbon(mut self, message: carbons::Received) -> Self {
        self.payloads.push(message.into());
        self
    }

    pub fn set_sent_carbon(mut self, message: carbons::Sent) -> Self {
        self.payloads.push(message.into());
        self
    }

    pub fn set_store(mut self, store: bool) -> Self {
        self.payloads.push(
            Element::builder(store.then_some("store").unwrap_or("no-store"), ns::HINTS).build(),
        );
        self
    }
}
