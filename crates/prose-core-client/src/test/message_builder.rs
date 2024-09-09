// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Duration, Utc};
use jid::BareJid;
use minidom::Element;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::{date, mam};

use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::{Forwarded, MucUser, Reactions};
use prose_xmpp::test::BareJidTestAdditions;

use crate::domain::messaging::models::{
    Body, Message, MessageFlags, MessageId, MessageLike, MessageLikeBody, MessageLikePayload,
    MessageRemoteId, MessageServerId, Reaction,
};
use crate::domain::shared::models::AnonOccupantId;
use crate::dtos::{
    Mention, Message as MessageDTO, MessageFlags as MessageFlagsDTO, MessageSender, ParticipantId,
    Reaction as ReactionDTO,
};
use crate::test::mock_data;

pub struct MessageBuilder {
    id: MessageId,
    remote_id: Option<MessageRemoteId>,
    stanza_id: Option<MessageServerId>,
    from: ParticipantId,
    from_anon: Option<AnonOccupantId>,
    from_name: Option<String>,
    to: BareJid,
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

    pub fn remote_id_for_index(idx: u32) -> MessageRemoteId {
        format!("msg-{}", idx).into()
    }

    pub fn stanza_id_for_index(idx: u32) -> MessageServerId {
        format!("res-{}", idx).into()
    }
}

impl MessageLikePayload {
    pub fn message(body: impl Into<String>) -> Self {
        let body = body.into();
        Self::Message {
            body: MessageLikeBody {
                raw: body.clone(),
                html: format!("<p>{}</p>", body).into(),
                mentions: vec![],
            },
            attachments: vec![],
            encryption_info: None,
            is_transient: false,
            reply_to: None,
        }
    }
}

impl<T> From<T> for MessageLikePayload
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        MessageLikePayload::message(value)
    }
}

impl MessageLikeBody {
    pub fn text(body: impl Into<String>) -> Self {
        let body = body.into();
        Self {
            raw: body.clone(),
            html: format!("<p>{}</p>", body).into(),
            mentions: vec![],
        }
    }

    pub fn with_mentions(mut self, mentions: impl IntoIterator<Item = Mention>) -> Self {
        self.mentions = mentions.into_iter().collect();
        self
    }
}

impl MessageBuilder {
    pub fn new_with_index(idx: u32) -> Self {
        Self::new_with_id(
            Self::id_for_index(idx),
            mock_data::reference_date() + Duration::minutes(idx.into()),
            MessageLikePayload::message(format!("Message {}", idx)),
        )
        .set_remote_id(Some(Self::remote_id_for_index(idx)))
        .set_server_id(Some(Self::stanza_id_for_index(idx)))
    }

    pub fn new_with_id(
        id: impl Into<MessageId>,
        timestamp: DateTime<Utc>,
        payload: MessageLikePayload,
    ) -> Self {
        MessageBuilder {
            id: id.into(),
            remote_id: None,
            stanza_id: None,
            from: ParticipantId::User(BareJid::ours().into()),
            from_anon: None,
            from_name: None,
            to: BareJid::theirs(),
            payload,
            timestamp,
            is_read: false,
            is_edited: false,
            is_delivered: false,
            reactions: vec![],
        }
    }

    pub fn set_id(mut self, id: impl Into<MessageId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn set_remote_id(mut self, remote_id: Option<MessageRemoteId>) -> Self {
        self.remote_id = remote_id;
        self
    }

    pub fn set_server_id(mut self, stanza_id: Option<MessageServerId>) -> Self {
        self.stanza_id = stanza_id;
        self
    }

    pub fn set_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn set_from(mut self, from: impl Into<ParticipantId>) -> Self {
        self.from = from.into();
        self
    }

    pub fn set_from_anon(mut self, from: impl Into<AnonOccupantId>) -> Self {
        self.from_anon = Some(from.into());
        self
    }

    pub fn set_from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    pub fn set_payload(mut self, payload: impl Into<MessageLikePayload>) -> Self {
        self.payload = payload.into();
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
            id: self.id,
            remote_id: self.remote_id,
            server_id: self.stanza_id,
            from: self.from,
            body: Body {
                raw: body.raw,
                html: body.html,
            },
            timestamp: self.timestamp.into(),
            flags: MessageFlags {
                is_read: self.is_read,
                is_edited: self.is_edited,
                is_delivered: self.is_delivered,
                is_transient: false,
                is_encrypted: false,
            },
            reactions: self.reactions,
            attachments: vec![],
            mentions: vec![],
            reply_to: None,
        }
    }

    pub fn build_message_dto(self) -> MessageDTO {
        let MessageLikePayload::Message { body, .. } = self.payload else {
            panic!("Cannot build MessageDTO from {:?}", self.payload);
        };

        MessageDTO {
            id: self.id,
            from: MessageSender {
                id: self.from,
                name: self
                    .from_name
                    .expect("You must set a name when building a MessageDTO"),
                avatar: None,
            },
            body: Body {
                raw: body.raw,
                html: body.html,
            },
            timestamp: self.timestamp.into(),
            flags: MessageFlagsDTO {
                is_read: self.is_read,
                is_edited: self.is_edited,
                is_delivered: self.is_delivered,
                is_transient: false,
                is_encrypted: false,
                is_last_read: false,
            },
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
                            avatar: None,
                        })
                        .collect(),
                })
                .collect(),
            attachments: vec![],
            mentions: vec![],
            reply_to: None,
        }
    }

    pub fn build_message_like(self) -> MessageLike {
        MessageLike {
            id: self.id,
            remote_id: self.remote_id,
            server_id: self.stanza_id,
            to: Some(self.to),
            from: self.from,
            timestamp: self.timestamp,
            payload: self.payload,
        }
    }

    pub fn build_reaction_to(self, target: u32, emoji: &[message::Emoji]) -> MessageLike {
        self.set_payload(MessageLikePayload::Reaction {
            target_id: Self::remote_id_for_index(target).into(),
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
        let stanza_id = self
            .stanza_id
            .clone()
            .expect("Missing stanzaId")
            .to_string()
            .into();
        let timestamp = date::DateTime(self.timestamp.clone().into());
        let mut message = self.build_message_stanza();

        if let Some(muc_user) = muc_user {
            message = message.set_muc_user(muc_user);
        }

        ArchivedMessage {
            id: stanza_id,
            query_id: Some(mam::QueryId(query_id.into())),
            forwarded: Forwarded {
                delay: Some(Delay {
                    from: None,
                    stamp: timestamp,
                    data: None,
                }),
                stanza: Some(Box::new(message)),
            },
        }
    }

    pub fn build_message_stanza(self) -> prose_xmpp::stanza::Message {
        let mut message = prose_xmpp::stanza::Message::new()
            .set_id(
                self.remote_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| self.id.to_string())
                    .into(),
            )
            .set_type(match self.from {
                ParticipantId::User(_) => MessageType::Chat,
                ParticipantId::Occupant(_) => MessageType::Groupchat,
            })
            .set_to(self.to)
            .set_from(self.from);

        if let Some(from_anon) = self.from_anon {
            message = message.add_payload(xmpp_parsers::occupant_id::OccupantId {
                id: from_anon.to_string(),
            });
        }

        match self.payload {
            MessageLikePayload::Error { message: error } => {
                message = message.set_body(format!("Error: {error}"))
            }
            MessageLikePayload::Message { body, .. } => message = message.set_body(body.raw),
            MessageLikePayload::Reaction { target_id, emojis } => {
                message = message.set_message_reactions(Reactions {
                    id: target_id.into_string(),
                    reactions: emojis.into_iter().map(Into::into).collect(),
                })
            }
            MessageLikePayload::Retraction { .. }
            | MessageLikePayload::Correction { .. }
            | MessageLikePayload::DeliveryReceipt { .. }
            | MessageLikePayload::ReadReceipt { .. } => {
                panic!("Cannot build ArchivedMessage from {:?}", self.payload)
            }
        }

        message
    }
}
