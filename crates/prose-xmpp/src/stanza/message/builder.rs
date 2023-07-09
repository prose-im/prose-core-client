use jid::Jid;
use xmpp_parsers::message::MessageType;

use crate::stanza::message::chat_marker::{Acknowledged, Displayed, Received};
use crate::stanza::message::fasten::ApplyTo;
use crate::stanza::message::mam::ArchivedMessage;
use crate::stanza::message::message::Message;
use crate::stanza::message::{ChatState, Fallback, Id, Reactions};

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
        self.id = Some(id);
        self
    }

    pub fn set_type(mut self, r#type: MessageType) -> Self {
        self.r#type = r#type;
        self
    }

    pub fn set_body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn set_chat_state(mut self, state: ChatState) -> Self {
        self.chat_state = Some(state);
        self
    }

    pub fn set_replace(mut self, id: Id) -> Self {
        self.replace = Some(id);
        self
    }

    pub fn set_message_reactions(mut self, reactions: Reactions) -> Self {
        self.reactions = Some(reactions);
        self
    }

    pub fn set_fastening(mut self, fastening: ApplyTo) -> Self {
        self.fastening = Some(fastening);
        self
    }

    pub fn set_fallback(mut self, fallback: Fallback) -> Self {
        self.fallback = Some(fallback);
        self
    }

    pub fn set_markable(mut self) -> Self {
        self.markable = true;
        self
    }

    pub fn set_received_marker(mut self, marker: Received) -> Self {
        self.received_marker = Some(marker);
        self
    }

    pub fn set_displayed_marker(mut self, marker: Displayed) -> Self {
        self.displayed_marker = Some(marker);
        self
    }

    pub fn set_acknowledged_marker(mut self, marker: Acknowledged) -> Self {
        self.acknowledged_marker = Some(marker);
        self
    }

    pub fn set_archived_message(mut self, message: ArchivedMessage) -> Self {
        self.archived_message = Some(message);
        self
    }
}
