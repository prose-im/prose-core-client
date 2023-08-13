// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{AccountBookmark, JID};
use anyhow::Result;
use std::path::Path;

pub struct AccountBookmarksClient {
    client: prose_core_client::AccountBookmarksClient,
}

impl AccountBookmarksClient {
    pub fn new(bookmarks_path: impl AsRef<Path>) -> Self {
        AccountBookmarksClient {
            client: prose_core_client::AccountBookmarksClient::new(bookmarks_path),
        }
    }

    pub fn load_bookmarks(&self) -> Result<Vec<AccountBookmark>> {
        Ok(self
            .client
            .load_bookmarks()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn add_bookmark(&self, jid: &JID, select_bookmark: bool) -> Result<()> {
        self.client
            .add_bookmark(&jid.clone().into(), select_bookmark)
    }

    pub fn remove_bookmark(&self, jid: &JID) -> Result<()> {
        self.client.remove_bookmark(&jid.clone().into())
    }

    pub fn select_bookmark(&self, jid: &JID) -> Result<()> {
        self.client.select_bookmark(&jid.clone().into())
    }

    pub fn save_bookmarks(&self, bookmarks: Vec<AccountBookmark>) -> Result<()> {
        self.client
            .save_bookmarks(bookmarks.into_iter().map(Into::into).collect())
    }
}
