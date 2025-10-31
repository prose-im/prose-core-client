// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use tracing::{error, info};

use prose_proc_macros::DependenciesStruct;
use prose_xmpp::TimeProvider;

use super::super::MessageArchiveDomainService as MessageArchiveDomainServiceTrait;
use crate::app::deps::{
    DynAppContext, DynEncryptionDomainService, DynLocalRoomSettingsRepository,
    DynMessageArchiveService, DynMessageIdProvider, DynMessagesRepository, DynTimeProvider,
};
use crate::domain::encryption::models::DecryptionContext;
use crate::domain::messaging::models::{MessageLike, MessageLikeError, MessageParser};
use crate::domain::messaging::services::MessagePage;
use crate::domain::rooms::models::Room;
use crate::dtos::{AccountId, MessageRemoteId, MessageServerId};
use crate::infra::xmpp::util::MessageExt;

#[derive(DependenciesStruct)]
pub struct MessageArchiveDomainService {
    ctx: DynAppContext,
    encryption_domain_service: DynEncryptionDomainService,
    local_room_settings_repo: DynLocalRoomSettingsRepository,
    message_archive_service: DynMessageArchiveService,
    message_id_provider: DynMessageIdProvider,
    message_repo: DynMessagesRepository,
    time_provider: DynTimeProvider,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageArchiveDomainServiceTrait for MessageArchiveDomainService {
    async fn catchup_room(&self, room: &Room, context: DecryptionContext) -> Result<bool> {
        if !room.features.is_mam_supported() {
            info!(
                "Skipping catchup on {} since it does not support MAM.",
                room.room_id
            );
            return Ok(false);
        }

        let account = self.ctx.connected_account()?;
        let connection_time = self.ctx.connection_timestamp()?;

        let last_catchup_time = self
            .local_room_settings_repo
            .get(&account, &room.room_id)
            .await?
            .last_catchup_time;

        // The idea here is that we want to catchup from either the last received message before
        // the current connection or from the last successful catchup.
        // We limit the last message to the last connection so that we don't consider offline
        // messages that we might have received upon connection.
        let last_received_message_time = self
            .message_repo
            .get_last_received_message(&account, &room.room_id, Some(connection_time))
            .await?
            .map(|message_ref| message_ref.timestamp);

        let catchup_since = last_catchup_time
            .max(last_received_message_time)
            .unwrap_or(DateTime::<Utc>::MIN_UTC)
            .max(
                self.time_provider.now()
                    - Duration::seconds(self.ctx.config.max_catchup_duration_secs),
            );

        info!("Catching up {} since {}", room.room_id, catchup_since);

        let mut messages = vec![];

        let page = self
            .message_archive_service
            .load_messages_since(&room.room_id, catchup_since, 100)
            .await?;

        let mut last_message_id = page
            .messages
            .last()
            .map(|m| MessageServerId::from(m.id.as_ref()));
        let mut is_last_page = page.is_last;

        self.parse_message_page(&account, room, page, &mut messages, &context)
            .await;

        while !is_last_page {
            let Some(message_id) = last_message_id.take() else {
                break;
            };

            let page = self
                .message_archive_service
                .load_messages_after(&room.room_id, &message_id, 100)
                .await?;

            last_message_id = page
                .messages
                .last()
                .map(|m| MessageServerId::from(m.id.as_ref()));
            is_last_page = page.is_last;

            self.parse_message_page(&account, room, page, &mut messages, &context)
                .await;
        }

        info!(
            "Finished catching up {}. Loaded {} messages.",
            room.room_id,
            messages.len()
        );
        self.message_repo
            .append(&account, &room.room_id, &messages)
            .await?;

        let now = self.time_provider.now();
        self.local_room_settings_repo
            .update(
                &account,
                &room.room_id,
                Box::new(move |settings| settings.last_catchup_time = Some(now)),
            )
            .await?;

        room.set_needs_update_statistics();

        let new_messages_found = !messages.is_empty();
        Ok(new_messages_found)
    }
}

impl MessageArchiveDomainService {
    async fn parse_message_page(
        &self,
        account: &AccountId,
        room: &Room,
        page: MessagePage,
        messages: &mut Vec<MessageLike>,
        context: &DecryptionContext,
    ) {
        for archive_message in page.messages {
            let inner_message = archive_message.forwarded.message.as_ref();

            let is_our_message = inner_message
                .sender()
                .map(|s| room.is_current_user(&account, &s.to_participant_id()))
                .unwrap_or_default();

            let message_id = if is_our_message {
                if let Some(remote_id) = archive_message.forwarded.message.id {
                    self.message_repo
                        .resolve_remote_id(
                            &account,
                            &room.room_id,
                            &MessageRemoteId::from(remote_id),
                        )
                        .await
                        .unwrap_or_default()
                        .map(|t| t.id)
                } else {
                    None
                }
            } else {
                self.message_repo
                    .resolve_server_id(
                        &account,
                        &room.room_id,
                        &MessageServerId::from(archive_message.id.as_ref()),
                    )
                    .await
                    .unwrap_or_default()
                    .map(|t| t.id)
            }
            .unwrap_or_else(|| self.message_id_provider.new_id());

            let parsed_message = match MessageParser::new(
                message_id,
                Some(room.clone()),
                Default::default(),
                self.encryption_domain_service.clone(),
                Some(context.clone()),
            )
            .parse_mam_message(archive_message)
            .await
            {
                Ok(message) => message,
                Err(error) => {
                    match error.downcast_ref::<MessageLikeError>() {
                        Some(MessageLikeError::NoPayload) => (),
                        None => {
                            error!("Failed to parse MAM message. {}", error.to_string());
                        }
                    }
                    continue;
                }
            };

            // Skip archived error messages. These usually don't have a message id, so the web
            // frontend chokes on that. And what's the point of archiving an error
            // message really?
            if parsed_message.payload.is_error() {
                continue;
            }

            messages.push(parsed_message)
        }
    }
}
