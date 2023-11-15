// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::sidebar::models::Bookmark;
use crate::domain::sidebar::services::BookmarksService;
use crate::infra::xmpp::type_conversions::bookmark::ns;
use crate::infra::xmpp::XMPPClient;
use anyhow::Result;
use prose_xmpp::mods;
use std::sync::Arc;

pub struct DebugService {
    client: Arc<XMPPClient>,
}

impl DebugService {
    pub fn new(client: Arc<XMPPClient>) -> Self {
        Self { client }
    }
}

impl DebugService {
    pub async fn delete_bookmarks_pubsub_node(&self) -> Result<()> {
        let pubsub = self.client.get_mod::<mods::PubSub>();
        pubsub.delete_node(ns::PROSE_BOOKMARK).await?;
        Ok(())
    }

    pub async fn load_bookmarks(&self) -> Result<Vec<Bookmark>> {
        self.client.load_bookmarks().await
    }
}
