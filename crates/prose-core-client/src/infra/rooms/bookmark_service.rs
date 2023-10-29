// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use xmpp_parsers::bookmarks2::{Autojoin, Conference};

use prose_xmpp::mods;
use prose_xmpp::stanza::ConferenceBookmark;

use crate::domain::rooms::models::Bookmark;
use crate::domain::rooms::services::BookmarksService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BookmarksService for XMPPClient {
    async fn load_bookmarks(&self) -> Result<Vec<Bookmark>> {
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        let bookmarks = bookmark_mod
            .load_bookmarks()
            .await?
            .into_iter()
            .map(|bookmark| Bookmark {
                name: bookmark.conference.name.unwrap_or(bookmark.jid.to_string()),
                room_jid: bookmark.jid.into_bare(),
            })
            .collect();
        Ok(bookmarks)
    }

    async fn publish_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<()> {
        let bookmark_mod = self.client.get_mod::<mods::Bookmark>();
        bookmark_mod
            .publish_bookmarks(bookmarks.iter().map(|bookmark| {
                ConferenceBookmark {
                    jid: Jid::Bare(bookmark.room_jid.clone()),
                    conference: Conference {
                        autojoin: Autojoin::True,
                        name: Some(bookmark.name.clone()),
                        // We're not saving a nickname so that we keep using the node of the
                        // logged-in user's JID instead.
                        nick: None,
                        password: None,
                        extensions: vec![],
                    },
                }
            }))
            .await?;
        Ok(())
    }
}
