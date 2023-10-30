// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::{date, mam, Element};

use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::test::BareJidTestAdditions;

use crate::domain::messaging::models::{
    Message, MessageId, MessageLike, MessageLikeId, MessageLikePayload, Reaction, StanzaId,
};
use crate::test::mock_data;

impl<T> From<T> for MessageLikeId
where
    T: Into<String>,
{
    fn from(s: T) -> MessageLikeId {
        MessageLikeId::from_str(&s.into()).unwrap()
    }
}

pub struct MessageBuilder {
    id: MessageId,
    stanza_id: Option<StanzaId>,
    from: BareJid,
    to: BareJid,
    body: String,
    timestamp: DateTime<Utc>,
    is_read: bool,
    is_edited: bool,
    is_delivered: bool,
    reactions: Vec<Reaction>,
    is_first_message: bool,
}

impl MessageBuilder {
    pub fn id_for_index(idx: u32) -> MessageId {
        format!("msg-{}", idx).into()
    }
}

impl MessageBuilder {
    pub fn new_with_index(idx: u32) -> Self {
        MessageBuilder {
            id: Self::id_for_index(idx),
            stanza_id: Some(format!("res-{}", idx).into()),
            from: BareJid::ours(),
            to: BareJid::theirs(),
            body: format!("Message {}", idx).to_string(),
            timestamp: mock_data::reference_date() + Duration::minutes(idx.into()),
            is_read: false,
            is_edited: false,
            is_delivered: false,
            reactions: vec![],
            is_first_message: false,
        }
    }

    pub fn set_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn set_from(mut self, from: &BareJid) -> Self {
        self.from = from.clone();
        self
    }
}

impl MessageBuilder {
    pub fn build_message(self) -> Message {
        Message {
            id: Some(self.id),
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
            id: MessageLikeId::new(Some(self.id)),
            stanza_id: self.stanza_id,
            target: None,
            to: Some(self.to),
            from: self.from,
            timestamp: self.timestamp,
            payload: MessageLikePayload::Message { body: self.body },
            is_first_message: self.is_first_message,
        }
    }

    pub fn build_message_like_with_payload(
        self,
        target: u32,
        payload: MessageLikePayload,
    ) -> MessageLike {
        MessageLike {
            id: MessageLikeId::new(Some(self.id)),
            stanza_id: self.stanza_id,
            target: Some(Self::id_for_index(target)),
            to: Some(self.to),
            from: self.from,
            timestamp: self.timestamp,
            payload,
            is_first_message: self.is_first_message,
        }
    }

    pub fn build_reaction_to(self, target: u32, emoji: &[message::Emoji]) -> MessageLike {
        self.build_message_like_with_payload(
            target,
            MessageLikePayload::Reaction {
                emojis: emoji.iter().cloned().collect(),
            },
        )
    }

    pub fn build_mam_message(self, query_id: impl Into<String>) -> Element {
        prose_xmpp::stanza::Message::new()
            .set_archived_message(ArchivedMessage {
                id: self.stanza_id.expect("Missing stanzaId").to_string().into(),
                query_id: Some(mam::QueryId(query_id.into())),
                forwarded: Forwarded {
                    delay: Some(Delay {
                        from: None,
                        stamp: date::DateTime(self.timestamp.into()),
                        data: None,
                    }),
                    stanza: Some(Box::new(
                        prose_xmpp::stanza::Message::new()
                            .set_id(self.id.as_ref().into())
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
