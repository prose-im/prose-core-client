use chrono::{DateTime, FixedOffset, Utc};
use jid::BareJid;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::{date, mam, Element};

use prose_core_domain::{Message, MessageId, Reaction, StanzaId};
use prose_core_lib::stanza::message::mam::ArchivedMessage;
use prose_core_lib::stanza::message::Forwarded;

use crate::test_helpers::{BareJidTestAdditions, DateTimeTestAdditions};
use crate::types::message_like::Payload;
use crate::types::MessageLike;

pub struct MessageBuilder {
    id: MessageId,
    stanza_id: Option<StanzaId>,
    from: BareJid,
    to: BareJid,
    body: String,
    timestamp: DateTime<FixedOffset>,
    is_read: bool,
    is_edited: bool,
    is_delivered: bool,
    reactions: Vec<Reaction>,
    is_first_message: bool,
}

impl MessageBuilder {
    pub fn new_with_index(idx: u32) -> Self {
        MessageBuilder {
            id: format!("msg-{}", idx).into(),
            stanza_id: Some(format!("res-{}", idx).into()),
            from: BareJid::ours(),
            to: BareJid::theirs(),
            body: format!("Message {}", idx).to_string(),
            timestamp: Utc::test_timestamp().into(),
            is_read: false,
            is_edited: false,
            is_delivered: false,
            reactions: vec![],
            is_first_message: false,
        }
    }

    pub fn set_timestamp(mut self, ts: DateTime<FixedOffset>) -> Self {
        self.timestamp = ts;
        self
    }
}

impl MessageBuilder {
    pub fn build_message(self) -> Message {
        Message {
            id: self.id,
            stanza_id: self.stanza_id,
            from: self.from,
            body: self.body,
            timestamp: self.timestamp.into(),
            is_read: self.is_read,
            is_edited: self.is_edited,
            is_delivered: self.is_delivered,
            reactions: self.reactions,
        }
    }

    pub fn build_message_like(self) -> MessageLike {
        MessageLike {
            id: self.id.0.into(),
            stanza_id: self.stanza_id.map(|id| id.0.into()),
            target: None,
            to: self.to,
            from: self.from,
            timestamp: self.timestamp,
            payload: Payload::Message { body: self.body },
            is_first_message: self.is_first_message,
        }
    }

    pub fn build_mam_message(self, query_id: impl Into<String>) -> Element {
        prose_core_lib::stanza::Message::new()
            .set_archived_message(ArchivedMessage {
                id: self.stanza_id.expect("Missing stanzaId").to_string().into(),
                query_id: Some(mam::QueryId(query_id.into())),
                forwarded: Forwarded {
                    delay: Some(Delay {
                        from: None,
                        stamp: date::DateTime(self.timestamp),
                        data: None,
                    }),
                    stanza: Some(Box::new(
                        prose_core_lib::stanza::Message::new()
                            .set_id(self.id.0.into())
                            .set_type(MessageType::Chat)
                            .set_to(self.to)
                            .set_from(self.from)
                            .set_body(self.body),
                    )),
                },
            })
            .into()
    }
}
