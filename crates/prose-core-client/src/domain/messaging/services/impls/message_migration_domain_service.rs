// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_proc_macros::DependenciesStruct;

use crate::app::deps::{DynMessageArchiveService, DynMessagingService};
use crate::domain::messaging::models::StanzaId;
use crate::domain::messaging::services::MessagePage;
use crate::domain::shared::models::RoomId;

use super::super::MessageMigrationDomainService as MessageMigrationDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct MessageMigrationDomainService {
    message_archive_service: DynMessageArchiveService,
    messaging_service: DynMessagingService,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl MessageMigrationDomainServiceTrait for MessageMigrationDomainService {
    async fn copy_all_messages_from_room(
        &self,
        source_room: &RoomId,
        target_room: &RoomId,
    ) -> Result<()> {
        let mut first_message_id: Option<StanzaId> = None;

        loop {
            let MessagePage { messages, is_last } = self
                .message_archive_service
                .load_messages(&source_room, first_message_id.as_ref(), None, 100)
                .await?;

            first_message_id = messages
                .first()
                .and_then(|m| m.forwarded.stanza.as_ref())
                .and_then(|m| m.stanza_id().clone())
                .map(|id| StanzaId::from(id.id.into_inner()));

            for message in messages {
                self.messaging_service
                    .relay_archived_message_to_room(target_room, message)
                    .await?;
            }

            if is_last {
                break;
            }
        }

        Ok(())
    }
}
