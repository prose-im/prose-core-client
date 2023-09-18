// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{UserMetadata, UserProfile};
use crate::CachePolicy;
use anyhow::Result;
use jid::BareJid;

pub struct DirectMessage;

impl<D: DataCache, A: AvatarCache> Room<DirectMessage, D, A> {
    pub async fn load_user_profile(&self) -> Result<Option<UserProfile>> {
        Ok(None)
    }

    pub async fn load_user_metadata(&self) -> Result<UserMetadata> {
        Ok(UserMetadata {
            local_time: None,
            last_activity: None,
        })
    }
}
