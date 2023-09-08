// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use tracing::instrument;

use prose_xmpp::mods::{Bookmark, Caps};
use prose_xmpp::stanza::ConferenceBookmark;

use crate::avatar_cache::AvatarCache;
use crate::client::muc;
use crate::data_cache::DataCache;

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    pub fn muc_service(&self) -> Option<muc::Service> {
        self.inner.muc_service.read().clone()
    }

    pub async fn load_bookmarks(&self) -> Result<Vec<ConferenceBookmark>> {
        let bookmark = self.client.get_mod::<Bookmark>();
        bookmark.load_bookmarks().await
    }

    pub async fn service_with_jid(&self, jid: &BareJid) -> Result<muc::Service> {
        Ok(muc::Service {
            jid: jid.clone(),
            user_jid: self.connected_jid()?.into_bare(),
            client: self.client.clone(),
        })
    }
}
