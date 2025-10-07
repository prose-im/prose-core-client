// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{types::ClientResult, AccountBookmark, PathBuf, JID};

#[derive(uniffi::Object)]
pub struct AccountBookmarksClient {
    client: prose_core_client::AccountBookmarksClient,
}

#[uniffi::export]
impl AccountBookmarksClient {
    #[uniffi::constructor]
    pub fn new(bookmarks_path: PathBuf) -> Self {
        AccountBookmarksClient {
            client: prose_core_client::AccountBookmarksClient::new(bookmarks_path.into_inner()),
        }
    }

    pub fn load_bookmarks(&self) -> ClientResult<Vec<AccountBookmark>> {
        Ok(self
            .client
            .load_bookmarks()?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn add_bookmark(&self, jid: &JID, select_bookmark: bool) -> ClientResult<()> {
        self.client
            .add_bookmark(&jid.clone().into(), select_bookmark)?;
        Ok(())
    }

    pub fn remove_bookmark(&self, jid: &JID) -> ClientResult<()> {
        self.client.remove_bookmark(&jid.clone().into())?;
        Ok(())
    }

    pub fn select_bookmark(&self, jid: &JID) -> ClientResult<()> {
        self.client.select_bookmark(&jid.clone().into())?;
        Ok(())
    }

    pub fn save_bookmarks(&self, bookmarks: Vec<AccountBookmark>) -> ClientResult<()> {
        self.client
            .save_bookmarks(bookmarks.into_iter().map(Into::into).collect())?;
        Ok(())
    }
}
