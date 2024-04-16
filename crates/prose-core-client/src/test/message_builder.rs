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
use prose_xmpp::stanza::message::{Forwarded, MucUser, Reactions};
use prose_xmpp::test::BareJidTestAdditions;

use crate::domain::messaging::models::{
    Message, MessageId, MessageLike, MessageLikeId, MessageLikePayload, Reaction, StanzaId,
};
use crate::dtos::{Message as MessageDTO, MessageSender, ParticipantId, Reaction as ReactionDTO};
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
    from: ParticipantId,
    from_name: Option<String>,
    to: BareJid,
    target_message_idx: Option<u32>,
    payload: MessageLikePayload,
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

    pub fn stanza_id_for_index(idx: u32) -> StanzaId {
        format!("res-{}", idx).into()
    }
}

impl MessageBuilder {
    pub fn new_with_index(idx: u32) -> Self {
        MessageBuilder {
            id: Self::id_for_index(idx),
            stanza_id: Some(Self::stanza_id_for_index(idx)),
            from: ParticipantId::User(BareJid::ours().into()),
            from_name: None,
            to: BareJid::theirs(),
            target_message_idx: None,
            payload: MessageLikePayload::Message {
                body: format!("Message {}", idx),
                attachments: vec![],
                mentions: vec![],
                encryption_info: None,
                is_transient: false,
            },
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

    pub fn set_from(mut self, from: impl Into<ParticipantId>) -> Self {
        self.from = from.into();
        self
    }

    pub fn set_from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    pub fn set_payload(mut self, payload: MessageLikePayload) -> Self {
        self.payload = payload;
        self
    }

    pub fn set_target_message_idx(mut self, idx: u32) -> Self {
        self.target_message_idx = Some(idx);
        self
    }

    pub fn set_reactions(mut self, reactions: impl IntoIterator<Item = Reaction>) -> Self {
        self.reactions = reactions.into_iter().collect();
        self
    }
}

impl MessageBuilder {
    pub fn build_message(self) -> Message {
        let MessageLikePayload::Message { body, .. } = self.payload else {
            panic!("Cannot build Message from {:?}", self.payload);
        };

        Message {
            id: Some(self.id),
            stanza_id: self.stanza_id,
            from: self.from,
            body,
            timestamp: self.timestamp.into(),
            is_read: self.is_read,
            is_edited: self.is_edited,
            is_delivered: self.is_delivered,
            is_transient: false,
            is_encrypted: false,
            reactions: self.reactions,
            attachments: vec![],
            mentions: vec![],
        }
    }

    pub fn build_message_dto(self) -> MessageDTO {
        let MessageLikePayload::Message { body, .. } = self.payload else {
            panic!("Cannot build MessageDTO from {:?}", self.payload);
        };

        MessageDTO {
            id: Some(self.id),
            stanza_id: self.stanza_id,
            from: MessageSender {
                id: self.from,
                name: self
                    .from_name
                    .expect("You must set a name when building a MessageDTO"),
            },
            body,
            timestamp: self.timestamp.into(),
            is_read: self.is_read,
            is_edited: self.is_edited,
            is_delivered: self.is_delivered,
            is_transient: false,
            is_encrypted: false,
            reactions: self
                .reactions
                .into_iter()
                .map(|reaction| ReactionDTO {
                    emoji: reaction.emoji,
                    from: reaction
                        .from
                        .into_iter()
                        .map(|sender| MessageSender {
                            id: sender.clone(),
                            name: sender
                                .to_user_id()
                                .map(|user_id| user_id.formatted_username())
                                .unwrap_or(sender.to_opaque_identifier()),
                        })
                        .collect(),
                })
                .collect(),
            attachments: vec![],
            mentions: vec![],
        }
    }

    pub fn build_message_like(self) -> MessageLike {
        MessageLike {
            id: MessageLikeId::new(Some(self.id)),
            stanza_id: self.stanza_id,
            target: self
                .target_message_idx
                .map(|idx| Self::id_for_index(idx).into()),
            to: Some(self.to),
            from: self.from,
            timestamp: self.timestamp,
            payload: self.payload,
        }
    }

    pub fn build_reaction_to(self, target: u32, emoji: &[message::Emoji]) -> MessageLike {
        self.set_target_message_idx(target)
            .set_payload(MessageLikePayload::Reaction {
                emojis: emoji.iter().cloned().collect(),
            })
            .build_message_like()
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
            .set_type(match self.from {
                ParticipantId::User(_) => MessageType::Chat,
                ParticipantId::Occupant(_) => MessageType::Groupchat,
            })
            .set_to(self.to)
            .set_from(self.from);

        match self.payload {
            MessageLikePayload::Error { message: error } => {
                message = message.set_body(format!("Error: {error}"))
            }
            MessageLikePayload::Message { body, .. } => message = message.set_body(body),
            MessageLikePayload::Reaction { emojis } => {
                message = message.set_message_reactions(Reactions {
                    id: Self::id_for_index(
                        self.target_message_idx.expect("Missing target_message_idx"),
                    )
                    .as_ref()
                    .into(),
                    reactions: emojis.into_iter().map(Into::into).collect(),
                })
            }
            MessageLikePayload::Retraction
            | MessageLikePayload::Correction { .. }
            | MessageLikePayload::DeliveryReceipt
            | MessageLikePayload::ReadReceipt => {
                panic!("Cannot build ArchivedMessage from {:?}", self.payload)
            }
        }

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
