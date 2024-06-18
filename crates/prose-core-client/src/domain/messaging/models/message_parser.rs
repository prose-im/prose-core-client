// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, SubsecRound, Utc};
use tracing::{error, warn};
use xmpp_parsers::message::MessageType;

use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::Forwarded;
use prose_xmpp::stanza::Message;

use crate::app::deps::DynEncryptionDomainService;
use crate::domain::encryption::models::DecryptionContext;
use crate::domain::messaging::models::message_like::Payload;
use crate::domain::messaging::models::{
    EncryptedMessage, MessageLike, MessageLikeEncryptionInfo, MessageLikeId, MessageTargetId,
    StanzaId, StanzaParseError,
};
use crate::domain::rooms::models::Room;
use crate::domain::shared::models::AnonOccupantId;
use crate::dtos::{DeviceId, Mention, MessageId, OccupantId, ParticipantId, RoomId, UserId};
use crate::infra::xmpp::type_conversions::stanza_error::StanzaErrorExt;
use crate::infra::xmpp::util::MessageExt;

pub struct MessageParser {
    room: Option<Room>,
    timestamp: DateTime<Utc>,
    encryption_domain_service: DynEncryptionDomainService,
    decryption_context: Option<DecryptionContext>,
}

impl MessageParser {
    pub fn new(
        room: Option<Room>,
        now: DateTime<Utc>,
        encryption_domain_service: DynEncryptionDomainService,
        decryption_context: Option<DecryptionContext>,
    ) -> Self {
        Self {
            room,
            timestamp: now.round_subsecs(0),
            encryption_domain_service,
            decryption_context,
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
        let (participant_id, user_id) = self.parse_sender(&message)?;
        // We're going to prefer the id of our associated room here, so that we'll even resolve
        // the correct id for sent messages where the from might be our JID.
        let room_id = self
            .room
            .as_ref()
            .map(|room| room.room_id.clone())
            .unwrap_or_else(|| participant_id.to_room_id());
        let TargetedPayload { target, payload } = self
            .parse_message_payload(user_id.as_ref(), &room_id, &message)
            .await?;
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
            from: user_id.map(ParticipantId::User).unwrap_or(participant_id),
            timestamp,
            payload,
        })
    }
}

impl MessageParser {
    fn parse_sender(
        &self,
        message: &Message,
    ) -> Result<(ParticipantId, Option<UserId>), StanzaParseError> {
        let Some(from) = &message.from else {
            return Err(StanzaParseError::missing_attribute("from"));
        };

        if message.is_groupchat_message() {
            let user_id = message
                .muc_user()
                .and_then(|muc_user| muc_user.jid)
                .map(|jid| jid.to_bare().into())
                .or_else(|| {
                    self.room
                        .as_ref()
                        .and_then(|room| {
                            message.occupant_id().map(|occupant_id| (room, occupant_id))
                        })
                        .and_then(|(room, occupant_id)| {
                            room.participants()
                                .get_user_id(&AnonOccupantId::from(occupant_id.id.clone()))
                        })
                });

            let from = from
                .try_as_full()
                .map_err(|_| StanzaParseError::ParseError {
                    error: "Expected `from` attribute to contain FullJid for groupchat message"
                        .to_string(),
                })?;
            Ok((
                ParticipantId::Occupant(OccupantId::from(from.clone())),
                user_id,
            ))
        } else {
            let user_id = UserId::from(from.to_bare());
            Ok((ParticipantId::User(user_id.clone()), Some(user_id)))
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
        sender_id: Option<&UserId>,
        room_id: &RoomId,
        message: &Message,
    ) -> Result<TargetedPayload> {
        if let Some(error) = &message.error() {
            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Error {
                    message: error.to_string(),
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

        // If the message doesn't have a body but does have attachments, we'll use an
        // empty string for the body.
        let parsed_body = self
            .parse_message_body(sender_id, room_id, message)
            .await?
            .or_else(|| {
                (!message.attachments().is_empty())
                    .then_some(ParsedMessageBody::Plaintext("".to_string()))
            });

        if let Some(parsed_body) = parsed_body {
            let (body, encryption_info) = match parsed_body {
                ParsedMessageBody::Plaintext(body) => (body, None),
                ParsedMessageBody::EncryptedMessage(body, info) => (body, Some(info)),
                ParsedMessageBody::EmptyMessage => return Err(MessageLikeError::NoPayload.into()),
            };

            let mentions = message
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
                .collect::<Vec<_>>();

            if let Some(replace_id) = message.replace() {
                return Ok(TargetedPayload {
                    target: Some(MessageTargetId::MessageId(replace_id.as_ref().into())),
                    payload: Payload::Correction {
                        body: body.to_string(),
                        attachments: message.attachments(),
                        mentions,
                        encryption_info,
                    },
                });
            }

            return Ok(TargetedPayload {
                target: None,
                payload: Payload::Message {
                    body: body.to_string(),
                    attachments: message.attachments(),
                    mentions,
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
        sender_id: Option<&UserId>,
        room_id: &RoomId,
        message: &Message,
    ) -> Result<Option<ParsedMessageBody>> {
        // If the message contains an encrypted payload, try to decrypt it. Otherwise, fall back
        // to the default body.
        if let (Some(sender_id), Some(omemo_element)) = (sender_id, message.omemo_element()) {
            let sender = DeviceId::from(omemo_element.header.sid);
            let encrypted_message = EncryptedMessage::from(omemo_element);
            let message_id = message.id.as_ref().map(|id| MessageId::from(id.clone()));

            let decryption_result = match encrypted_message {
                EncryptedMessage::Message(message) => {
                    self.encryption_domain_service
                        .decrypt_message(
                            sender_id,
                            room_id,
                            message_id.as_ref(),
                            message,
                            self.decryption_context.clone(),
                        )
                        .await
                }
                EncryptedMessage::KeyTransport(payload) => {
                    let res = self
                        .encryption_domain_service
                        .handle_received_key_transport_message(
                            sender_id,
                            payload,
                            self.decryption_context.clone(),
                        )
                        .await;
                    if let Err(error) = res {
                        error!(
                            "Failed to handle KeyTransportMessage from {sender_id}. {}",
                            error.to_string()
                        );
                    };
                    return Ok(Some(ParsedMessageBody::EmptyMessage));
                }
            };

            let parsed_message = match decryption_result {
                Ok(body) => {
                    ParsedMessageBody::EncryptedMessage(body, MessageLikeEncryptionInfo { sender })
                }
                Err(error) => {
                    error!(
                        "Failed to decrypt message from {sender_id}. {}",
                        error.to_string()
                    );
                    ParsedMessageBody::EncryptedMessage(
                        message
                            .body()
                            .unwrap_or(
                                "Message failed to decrypt and did not contain a fallback text.",
                            )
                            .to_string(),
                        MessageLikeEncryptionInfo { sender },
                    )
                }
            };
            return Ok(Some(parsed_message));
        }

        Ok(message
            .body()
            .map(|body| ParsedMessageBody::Plaintext(body.to_string())))
    }
}

/// Represents the body of the message.
enum ParsedMessageBody {
    /// The message was sent unencrypted.
    Plaintext(String),
    /// The message was sent encrypted.
    EncryptedMessage(String, MessageLikeEncryptionInfo),
    /// The message did not contain a human-readable body. This can happen for messages that are
    /// used to exchange OMEMO key material.
    EmptyMessage,
}
