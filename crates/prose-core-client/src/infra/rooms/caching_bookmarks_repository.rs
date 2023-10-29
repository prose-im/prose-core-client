// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::{HashMap, HashSet};

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use parking_lot::RwLock;

use crate::app::deps::DynBookmarksService;
use crate::domain::rooms::models::Bookmark;
use crate::domain::rooms::repos::BookmarksRepository;

pub struct CachingBookmarksRepository {
    service: DynBookmarksService,
    bookmarks: RwLock<Option<HashMap<BareJid, Bookmark>>>,
}

impl CachingBookmarksRepository {
    pub fn new(service: DynBookmarksService) -> Self {
        Self {
            service,
            bookmarks: Default::default(),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BookmarksRepository for CachingBookmarksRepository {
    async fn get_all(&self) -> Result<Vec<Bookmark>> {
        self.fetch_bookmarks_if_needed().await?;
        Ok(self
            .bookmarks
            .read()
            .clone()
            .map(|bookmarks| bookmarks.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default())
    }

    async fn put(&self, bookmark: Bookmark) -> Result<()> {
        self.modify_and_publish_bookmarks(|bookmarks| {
            bookmarks.insert(bookmark.room_jid.clone(), bookmark);
        })
        .await?;
        Ok(())
    }

    async fn delete(&self, room_jids: &[BareJid]) -> Result<()> {
        self.fetch_bookmarks_if_needed().await?;

        let room_jids = room_jids.iter().collect::<HashSet<&BareJid>>();
        self.modify_and_publish_bookmarks(|bookmarks| {
            bookmarks.retain(|room_jid, _| !room_jids.contains(room_jid));
        })
        .await?;
        Ok(())
    }
}

impl CachingBookmarksRepository {
    async fn fetch_bookmarks_if_needed(&self) -> Result<()> {
        if self.bookmarks.read().is_some() {
            return Ok(());
        }
        let bookmarks = self
            .service
            .load_bookmarks()
            .await?
            .into_iter()
            .map(|bookmark| (bookmark.room_jid.clone(), bookmark))
            .collect::<HashMap<_, _>>();
        self.bookmarks.write().replace(bookmarks);
        Ok(())
    }

    async fn modify_and_publish_bookmarks<F>(&self, handler: F) -> Result<()>
    where
        F: FnOnce(&mut HashMap<BareJid, Bookmark>),
    {
        self.fetch_bookmarks_if_needed().await?;

        let bookmarks = self.bookmarks.write().take().unwrap_or(Default::default());
        let mut mutated_bookmarks = bookmarks.clone();

        (handler)(&mut mutated_bookmarks);

        if bookmarks == mutated_bookmarks {
            self.bookmarks.write().replace(bookmarks);
            return Ok(());
        }

        let bookmarks = mutated_bookmarks.values().cloned().collect::<Vec<_>>();
        self.bookmarks.write().replace(mutated_bookmarks);

        self.service.publish_bookmarks(bookmarks.as_slice()).await?;

        Ok(())
    }
}
