// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::instrument;

use prose_xmpp::mods::bookmark::ConferenceBookmark;
use prose_xmpp::mods::{Bookmark, Caps};

use crate::avatar_cache::AvatarCache;
use crate::client::muc;
use crate::data_cache::DataCache;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn load_muc_services(&self) -> Result<Vec<muc::Service>> {
        let caps = self.client.get_mod::<Caps>();
        let disco_items = caps.query_server_disco_items(None).await?.items;
        let mut services = vec![];

        for item in disco_items {
            let info = caps.query_disco_info(item.jid.clone(), None).await?;

            if info
                .identities
                .iter()
                .find(|ident| ident.category == "conference")
                .is_none()
            {
                continue;
            }

            services.push(muc::Service {
                client: self.client.clone(),
                jid: item.jid.into_bare(),
            });
        }

        Ok(services)
    }

    pub async fn load_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let bookmark = self.client.get_mod::<Bookmark>();
        bookmark.load_bookmarks().await
    }
}
