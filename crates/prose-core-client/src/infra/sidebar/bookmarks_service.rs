// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use minidom::Element;
use xmpp_parsers::data_forms::{Field, FieldType};
use xmpp_parsers::pubsub::pubsub::PublishOptions;
use xmpp_parsers::pubsub::{Item, ItemId};

use prose_xmpp::{mods, PublishOptionsExt, RequestError};

use crate::domain::sidebar::models::Bookmark;
use crate::domain::sidebar::services::BookmarksService;
use crate::infra::xmpp::type_conversions::bookmark::ns;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BookmarksService for XMPPClient {
    async fn load_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let pubsub = self.client.get_mod::<mods::PubSub>();
        let bookmarks = pubsub
            .load_all_items(ns::PROSE_BOOKMARK)
            .await?
            .into_iter()
            .map(|item| {
                let Some(payload) = item.payload else {
                    return Err(RequestError::UnexpectedResponse.into());
                };
                Bookmark::try_from(payload)
            })
            .collect::<Result<Vec<_>>>();

        Ok(bookmarks?)
    }

    async fn save_bookmark(&self, bookmark: &Bookmark) -> Result<()> {
        let item = Item {
            id: Some(ItemId(bookmark.jid.to_string())),
            publisher: None,
            payload: Some(Element::from(bookmark.clone())),
        };

        let pubsub = self.client.get_mod::<mods::PubSub>();
        pubsub
            .publish_items(
                ns::PROSE_BOOKMARK,
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

    async fn delete_bookmark(&self, jid: &BareJid) -> Result<()> {
        let pubsub = self.client.get_mod::<mods::PubSub>();
        pubsub
            .delete_items_with_ids(ns::PROSE_BOOKMARK, [jid.to_string()])
            .await?;
        Ok(())
    }
}
