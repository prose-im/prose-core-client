// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::domain::shared::models::UserId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AccountBookmark {
    pub user_id: UserId,
    #[serde(rename = "selected")]
    pub is_selected: bool,
}

impl AccountBookmark {
    pub fn new(user_id: UserId, is_selected: bool) -> Self {
        AccountBookmark {
            user_id,
            is_selected,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AccountBookmarksFile {
    accounts: Vec<AccountBookmark>,
}

pub struct AccountBookmarksClient {
    bookmarks_path: PathBuf,
}

impl AccountBookmarksClient {
    pub fn new(bookmarks_path: impl AsRef<Path>) -> Self {
        AccountBookmarksClient {
            bookmarks_path: bookmarks_path.as_ref().to_path_buf(),
        }
    }
}

impl AccountBookmarksClient {
    pub fn load_bookmarks(&self) -> Result<Vec<AccountBookmark>> {
        if !self.bookmarks_path.exists() {
            return Ok(vec![]);
        }
        let file = File::open(&self.bookmarks_path)?;
        let contents: AccountBookmarksFile = serde_json::from_reader(BufReader::new(file))?;
        Ok(contents.accounts)
    }

    pub fn add_bookmark(&self, user_id: &UserId, select_bookmark: bool) -> Result<()> {
        let mut bookmarks = self.load_bookmarks()?;

        if !bookmarks
            .iter()
            .any(|bookmark| &bookmark.user_id == user_id)
        {
            bookmarks.push(AccountBookmark {
                user_id: user_id.clone(),
                is_selected: false,
            });
        }

        if select_bookmark || bookmarks.len() == 1 {
            for bookmark in bookmarks.iter_mut() {
                bookmark.is_selected = &bookmark.user_id == user_id;
            }
        }
        self.save_bookmarks(bookmarks)
    }

    pub fn remove_bookmark(&self, user_id: &UserId) -> Result<()> {
        let mut bookmarks = self.load_bookmarks()?;
        bookmarks.retain(|bookmark| &bookmark.user_id != user_id);
        if !bookmarks.iter().any(|bookmark| bookmark.is_selected) {
            if let Some(bookmark) = bookmarks.get_mut(0) {
                bookmark.is_selected = true;
            }
        }
        self.save_bookmarks(bookmarks)
    }

    pub fn select_bookmark(&self, user_id: &UserId) -> Result<()> {
        let mut bookmarks = self.load_bookmarks()?;
        for bookmark in bookmarks.iter_mut() {
            bookmark.is_selected = &bookmark.user_id == user_id;
        }
        self.save_bookmarks(bookmarks)?;
        Ok(())
    }
}

impl AccountBookmarksClient {
    pub fn save_bookmarks(&self, bookmarks: Vec<AccountBookmark>) -> Result<()> {
        let file = NamedTempFile::new()?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(
            &mut writer,
            &AccountBookmarksFile {
                accounts: bookmarks,
            },
        )?;
        writer.write(b"\n")?;
        writer.flush()?;
        writer.into_inner()?.persist(&self.bookmarks_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_save_and_load_bookmarks() -> Result<()> {
        let mut path = temp_dir();
        path.push("bookmarks.json");

        if path.exists() {
            fs::remove_file(&path)?;
        }

        println!("{:?}", path);

        let a = UserId::from_str("a@prose.org")?;
        let b = UserId::from_str("b@prose.org")?;
        let c = UserId::from_str("c@prose.org")?;

        let client = AccountBookmarksClient::new(path.clone());
        client.add_bookmark(&a, false)?;

        assert_eq!(
            client.load_bookmarks()?,
            [AccountBookmark::new(a.clone(), true)]
        );

        client.add_bookmark(&b, false)?;
        assert_eq!(
            client.load_bookmarks()?,
            [
                AccountBookmark::new(a.clone(), true),
                AccountBookmark::new(b.clone(), false)
            ]
        );

        client.add_bookmark(&c, true)?;
        assert_eq!(
            client.load_bookmarks()?,
            [
                AccountBookmark::new(a.clone(), false),
                AccountBookmark::new(b.clone(), false),
                AccountBookmark::new(c.clone(), true),
            ]
        );

        client.add_bookmark(&c, true)?;
        assert_eq!(
            client.load_bookmarks()?,
            [
                AccountBookmark::new(a.clone(), false),
                AccountBookmark::new(b.clone(), false),
                AccountBookmark::new(c.clone(), true),
            ]
        );

        client.select_bookmark(&b)?;
        assert_eq!(
            client.load_bookmarks()?,
            [
                AccountBookmark::new(a.clone(), false),
                AccountBookmark::new(b.clone(), true),
                AccountBookmark::new(c.clone(), false),
            ]
        );

        client.remove_bookmark(&b)?;
        assert_eq!(
            client.load_bookmarks()?,
            [
                AccountBookmark::new(a.clone(), true),
                AccountBookmark::new(c.clone(), false),
            ]
        );

        client.remove_bookmark(&a)?;
        assert_eq!(
            client.load_bookmarks()?,
            [AccountBookmark::new(c.clone(), true),]
        );

        client.remove_bookmark(&c)?;
        assert_eq!(client.load_bookmarks()?, []);

        Ok(())
    }
}
