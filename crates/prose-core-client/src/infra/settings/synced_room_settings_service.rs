// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use minidom::Element;
use xmpp_parsers::data_forms::{Field, FieldType};
use xmpp_parsers::pubsub::pubsub::PublishOptions;
use xmpp_parsers::pubsub::{Item, ItemId};

use prose_xmpp::{mods, PublishOptionsExt};

use crate::domain::settings::models::SyncedRoomSettings;
use crate::domain::settings::services::SyncedRoomSettingsService;
use crate::domain::shared::models::RoomId;
use crate::infra::xmpp::type_conversions::synced_room_settings::ns;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl SyncedRoomSettingsService for XMPPClient {
    async fn load_settings(&self, room_id: &RoomId) -> Result<Option<SyncedRoomSettings>> {
        let pubsub = self.client.get_mod::<mods::PubSub>();
        let mut settings = pubsub
            .load_objects_with_ids::<SyncedRoomSettings, _>(
                ns::PROSE_ROOM_SETTINGS,
                [room_id.to_string()],
            )
            .await?;
        Ok(settings.pop())
    }

    async fn save_settings(&self, room_id: &RoomId, settings: &SyncedRoomSettings) -> Result<()> {
        let item = Item {
            id: Some(ItemId(room_id.to_string())),
            publisher: None,
            payload: Some(Element::from(settings.clone())),
        };

        let pubsub = self.client.get_mod::<mods::PubSub>();
        pubsub
            .publish_items(
                ns::PROSE_ROOM_SETTINGS,
                [item],
                Some(PublishOptions::for_private_data([
                    Field::new("pubsub#max_items", FieldType::TextSingle).with_value("256"),
                    Field::new("pubsub#send_last_published_item", FieldType::ListSingle)
                        .with_value("never"),
                ])),
            )
            .await?;
        Ok(())
    }

    async fn delete_settings(&self, room_id: &RoomId) -> Result<()> {
        let pubsub = self.client.get_mod::<mods::PubSub>();
        pubsub
            .delete_items_with_ids(ns::PROSE_ROOM_SETTINGS, [room_id.to_string()], true)
            .await?;
        Ok(())
    }
}
