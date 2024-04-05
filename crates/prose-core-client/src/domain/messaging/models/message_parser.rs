// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, Utc};
use jid::Jid;
use tracing::{error, warn};
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::stanza::Message;

use crate::app::deps::DynEncryptionDomainService;
use crate::domain::messaging::models::message_like::Payload;
use crate::domain::messaging::models::{
    MessageLike, MessageLikeEncryptionInfo, MessageLikeId, MessageTargetId, StanzaId,
    StanzaParseError,
};
use crate::dtos::{DeviceId, Mention, MessageId, OccupantId, ParticipantId, UserId};
use crate::infra::xmpp::type_conversions::stanza_error::StanzaErrorExt;
use crate::infra::xmpp::util::MessageExt;

pub struct MessageParser {
    timestamp: DateTime<Utc>,
    encryption_domain_service: DynEncryptionDomainService,
}

impl MessageParser {
    pub fn new(now: DateTime<Utc>, encryption_domain_service: DynEncryptionDomainService) -> Self {
        Self {
            timestamp: now,
            encryption_domain_service,
        }
    }
}

impl MessageParser {
    pub async fn parse_mam_message(self, mam_message: ArchivedMessage) -> Result<MessageLike> {
        let mut parsed_message = self.parse_forwarded_message(mam_message.forwarded).await?;
        parsed_message.stanza_id = Some(StanzaId::from(mam_message.id.into_inner()));
        Ok(parsed_message)
    }

    pub async fn parse_carbon(self, carbon: Carbon) -> Result<MessageLike> {
        let forwarded = match carbon {
            Carbon::Received(carbon) => carbon,
            Carbon::Sent(carbon) => carbon,
        };
        self.parse_forwarded_message(forwarded).await
    }

    pub async fn parse_forwarded_message(
        self,
        forwarded_message: Forwarded,
    ) -> Result<MessageLike> {
        let mut parsed_message = self
            .parse_message(
                *forwarded_message
                    .stanza
                    .ok_or(StanzaParseError::missing_child_node("message"))?,
            )
            .await?;

        if let Some(delay) = forwarded_message.delay {
            parsed_message.timestamp = delay.stamp.0.into()
        }

        Ok(parsed_message)
    }

    pub async fn parse_message(self, message: Message) -> Result<MessageLike> {
        let stanza_id = message
            .stanza_id()
            .map(|sid| StanzaId::from(sid.id.into_inner()));
        let from = self.parse_from(&message)?;
        let TargetedPayload { target, payload } =
            self.parse_message_payload(&from, &message).await?;
        let timestamp = message
            .delay()
            .map(|delay| delay.stamp.0.into())
            .unwrap_or(self.timestamp);

        let message = message.into_inner();
        let id = MessageLikeId::new(message.id.map(Into::into));
        let to = message.to.map(|jid| jid.into_bare());

        Ok(MessageLike {
            id,
            stanza_id,
            target,
            to,
            from,
            timestamp,
            payload,
        })
    }
}

impl MessageParser {
    fn parse_from(&self, message: &Message) -> Result<ParticipantId, StanzaParseError> {
        let Some(from) = &message.from else {
            return Err(StanzaParseError::missing_attribute("from"));
        };

        if message.is_groupchat_message() {
            if let Some(muc_user) = &message.muc_user() {
                if let Some(jid) = &muc_user.jid {
                    return Ok(ParticipantId::User(jid.to_bare().into()));
                }
            }
            let Jid::Full(from) = from else {
                return Err(StanzaParseError::ParseError {
                    error: "Expected `from` attribute to contain FullJid for groupchat message"
                        .to_string(),
                });
            };
            Ok(ParticipantId::Occupant(OccupantId::from(from.clone())))
        } else {
            Ok(ParticipantId::User(UserId::from(from.to_bare())))
        }
    }
}

struct TargetedPayload {
    target: Option<MessageTargetId>,
    payload: Payload,
}

#[derive(thiserror::Error, Debug)]
pub enum MessageLikeError {
    #[error("No payload in message")]
    NoPayload,
}

impl MessageParser {
    async fn parse_message_payload(
        &self,
        from: &ParticipantId,
        message: &Message,
    ) -> Result<TargetedPayload> {
        if let Some(error) = &message.error() {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message {
                    body: format!("Error: {}", error.to_string()),
                    attachments: vec![],
                    mentions: vec![],
                    encryption_info: None,
                    is_transient: false,
                },
            });
        }

        let is_groupchat_message = message.is_groupchat_message();

        if let Some(reactions) = message.reactions() {
            return Ok(TargetedPayload {
                target: Some(if is_groupchat_message {
                    MessageTargetId::StanzaId(reactions.id.into())
                } else {
                    MessageTargetId::MessageId(reactions.id.into())
                }),
                payload: Payload::Reaction {
                    emojis: reactions.reactions,
                },
            });
        };

        if let Some(fastening) = message.fastening() {
            if fastening.retract() {
                return Ok(TargetedPayload {
                    target: Some(MessageTargetId::MessageId(fastening.id.as_ref().into())),
                    payload: Payload::Retraction,
                });
            }
        }

        if let Some(marker) = message.received_marker() {
            return Ok(TargetedPayload {
                target: Some(if is_groupchat_message {
                    MessageTargetId::StanzaId(marker.id.as_ref().into())
                } else {
                    MessageTargetId::MessageId(marker.id.as_ref().into())
                }),
                payload: Payload::DeliveryReceipt,
            });
        }

        if let Some(marker) = message.displayed_marker() {
            return Ok(TargetedPayload {
                target: Some(if is_groupchat_message {
                    MessageTargetId::StanzaId(marker.id.as_ref().into())
                } else {
                    MessageTargetId::MessageId(marker.id.as_ref().into())
                }),
                payload: Payload::ReadReceipt,
            });
        }

        if let Some((body, encryption_info)) = self.parse_message_body(from, message).await? {
            if let Some(replace_id) = message.replace() {
                return Ok(TargetedPayload {
                    target: Some(MessageTargetId::MessageId(replace_id.as_ref().into())),
                    payload: Payload::Correction {
                        body: body.to_string(),
                        attachments: message.attachments(),
                        encryption_info,
                    },
                });
            }

            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message {
                    body: body.to_string(),
                    attachments: message.attachments(),
                    mentions: message
                        .mentions()
                        .into_iter()
                        .filter_map(|r| match Mention::try_from(r) {
                            Ok(mention) => Some(mention),
                            Err(err) => {
                                warn!(
                                    "Failed to parse mention from reference. {}",
                                    err.to_string()
                                );
                                None
                            }
                        })
                        .collect(),
                    encryption_info,
                    // A message that we consider a groupchat message but is of type 'chat' is
                    // usually a private message. We'll treat them as transient messages.
                    is_transient: is_groupchat_message && message.type_ == MessageType::Chat,
                },
            });
        }

        error!("Failed to parse message {:?}", message);

        Err(MessageLikeError::NoPayload.into())
    }

    async fn parse_message_body(
        &self,
        from: &ParticipantId,
        message: &Message,
    ) -> Result<Option<(String, Option<MessageLikeEncryptionInfo>)>> {
        // If the message contains an encrypted payload, try to decrypt it. Otherwise, fall back
        // to the default body.
        if let (ParticipantId::User(sender_id), Some(message_id), Some(omemo_element)) =
            (from, &message.id, message.omemo_element())
        {
            let sender = DeviceId::from(omemo_element.header.sid);

            if let Ok(body) = self
                .encryption_domain_service
                .decrypt_message(
                    sender_id,
                    &MessageId::from(message_id),
                    omemo_element.into(),
                )
                .await
            {
                return Ok(Some((body, Some(MessageLikeEncryptionInfo { sender }))));
            }
        }

        Ok(message.body().map(|body| {
            (
                body.to_string(),
                message
                    .omemo_element()
                    .map(|omemo| MessageLikeEncryptionInfo {
                        sender: omemo.header.sid.into(),
                    }),
            )
        }))
    }
}
