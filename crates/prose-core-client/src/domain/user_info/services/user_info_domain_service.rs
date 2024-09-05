// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::contacts::models::Contact;
use crate::domain::shared::models::{CachePolicy, UserId, UserOrResourceId};
use crate::domain::user_info::models::{
    Avatar, PlatformImage, Presence, UserInfo, UserMetadata, UserProfile, UserStatus,
};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn get_user_info(
        &self,
        user_id: &UserId,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserInfo>>;

    async fn get_user_profile(
        &self,
        user_id: &UserId,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserProfile>>;

    async fn get_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>>;

    async fn load_avatar_image(&self, avatar: &Avatar) -> Result<Option<PlatformImage>>;

    async fn handle_user_presence_changed(
        &self,
        user_id: &UserOrResourceId,
        presence: Presence,
    ) -> Result<()>;

    async fn handle_user_status_changed(
        &self,
        user_id: &UserId,
        user_activity: Option<UserStatus>,
    ) -> Result<()>;

    async fn handle_avatar_changed(&self, user_id: &UserId, avatar: Option<Avatar>) -> Result<()>;

    async fn handle_user_profile_changed(
        &self,
        user_id: &UserId,
        profile: Option<UserProfile>,
    ) -> Result<()>;

    async fn handle_nickname_changed(
        &self,
        user_id: &UserId,
        nickname: Option<String>,
    ) -> Result<()>;

    async fn handle_contacts_changed(&self, contacts: Vec<Contact>) -> Result<()>;

    async fn reset_before_reconnect(&self) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
