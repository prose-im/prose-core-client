// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use tracing::info;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAppContext, DynClientEventDispatcher, DynConnectedRoomsReadOnlyRepository,
};
use crate::app::event_handlers::{
    PubSubEventType, ServerEvent, ServerEventHandler, SyncedRoomSettingsEvent,
};
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct SyncedRoomSettingsEventHandler {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl ServerEventHandler for SyncedRoomSettingsEventHandler {
    fn name(&self) -> &'static str {
        "remote-room-settings"
    }

    async fn handle_event(&self, event: ServerEvent) -> Result<Option<ServerEvent>> {
        match event {
            ServerEvent::SyncedRoomSettings(event) => {
                self.handle_synced_room_settings_event(event).await?
            }
            _ => return Ok(Some(event)),
        }

        Ok(None)
    }
}

impl SyncedRoomSettingsEventHandler {
    async fn handle_synced_room_settings_event(
        &self,
        event: SyncedRoomSettingsEvent,
    ) -> Result<()> {
        match event.r#type {
            PubSubEventType::AddedOrUpdated { items: settings } => {
                for setting in settings {
                    let Some(room) = self.connected_rooms_repo.get(
                        &self.ctx.connected_account()?,
                        &setting.room_id.clone().into_bare(),
                    ) else {
                        continue;
                    };

                    info!("Applying updated room settings in {}", room.room_id);
                    *room.settings_mut() = setting;
                    room.set_needs_update_statistics();
                }
                self.client_event_dispatcher
                    .dispatch_event(ClientEvent::SidebarChanged);
            }
            PubSubEventType::Deleted { .. } | PubSubEventType::Purged => (),
        }

        Ok(())
    }
}
