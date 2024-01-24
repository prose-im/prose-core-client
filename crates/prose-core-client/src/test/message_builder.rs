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
use prose_xmpp::stanza::message::{Forwarded, MucUser};
use prose_xmpp::test::BareJidTestAdditions;

use crate::domain::messaging::models::{
    Message, MessageId, MessageLike, MessageLikeId, MessageLikePayload, Reaction, StanzaId,
};
use crate::domain::shared::models::UserEndpointId;
use crate::dtos::{Message as MessageDTO, MessageSender};
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
    from: UserEndpointId,
    from_name: Option<String>,
    to: BareJid,
    body: String,
    timestamp: DateTime<Utc>,
    is_read: bool,
    is_edited: bool,
    is_delivered: bool,
    reactions: Vec<Reaction>,
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
            from: UserEndpointId::User(BareJid::ours().into()),
            from_name: None,
            to: BareJid::theirs(),
            body: format!("Message {}", idx).to_string(),
            timestamp: mock_data::reference_date() + Duration::minutes(idx.into()),
            is_read: false,
            is_edited: false,
            is_delivered: false,
            reactions: vec![],
        }
    }

    pub fn set_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn set_from(mut self, from: impl Into<UserEndpointId>) -> Self {
        self.from = from.into();
        self
    }

    pub fn set_from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }
}

impl MessageBuilder {
    pub fn build_message(self) -> Message {
        Message {
            id: Some(self.id),
            stanza_id: self.stanza_id,
            from: self.from.into_jid(),
            body: self.body,
            timestamp: self.timestamp.into(),
            is_read: self.is_read,
            is_edited: self.is_edited,
            is_delivered: self.is_delivered,
            reactions: self.reactions,
        }
    }

    pub fn build_message_dto(self) -> MessageDTO {
        MessageDTO {
            id: Some(self.id),
            stanza_id: self.stanza_id,
            from: MessageSender {
                id: self.from.to_user_id(),
                name: self
                    .from_name
                    .expect("You must set a name when building a MessageDTO"),
            },
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
            from: self.from.into_jid(),
            timestamp: self.timestamp,
            payload: MessageLikePayload::Message { body: self.body },
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
            from: self.from.into_jid(),
            timestamp: self.timestamp,
            payload,
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

    pub fn build_mam_message(
        self,
        query_id: impl Into<String>,
        muc_user: Option<MucUser>,
    ) -> Element {
        prose_xmpp::stanza::Message::new()
            .set_archived_message(self.build_archived_message(query_id, muc_user))
            .into()
    }

    pub fn build_archived_message(
        self,
        query_id: impl Into<String>,
        muc_user: Option<MucUser>,
    ) -> ArchivedMessage {
        let mut message = prose_xmpp::stanza::Message::new()
            .set_id(self.id.as_ref().into())
            .set_type(MessageType::Chat)
            .set_to(self.to)
            .set_from(self.from.into_jid())
            .set_body(self.body);

        if let Some(muc_user) = muc_user {
            message = message.set_muc_user(muc_user);
        }

        ArchivedMessage {
            id: self.stanza_id.expect("Missing stanzaId").to_string().into(),
            query_id: Some(mam::QueryId(query_id.into())),
            forwarded: Forwarded {
                delay: Some(Delay {
                    from: None,
                    stamp: date::DateTime(self.timestamp.into()),
                    data: None,
                }),
                stanza: Some(Box::new(message)),
            },
        }
    }
}
