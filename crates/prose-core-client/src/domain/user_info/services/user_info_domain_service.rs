// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::shared::models::UserOrResourceId;
use crate::domain::user_info::models::{AvatarMetadata, Presence};
use crate::dtos::{UserId, UserInfo, UserMetadata, UserProfile, UserStatus};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserInfoDomainService: SendUnlessWasm + SyncUnlessWasm {
    /// Returns the display name for `user_id`. Display name is a cascade of first_name, last_name
    /// and nickname;
    async fn get_display_name(&self, user_id: &UserId) -> Result<Option<String>>;

    async fn get_user_info(&self, user_id: &UserId) -> Result<Option<UserInfo>>;

    async fn get_user_profile(&self, user_id: &UserId) -> Result<Option<UserProfile>>;

    async fn get_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>>;

    async fn handle_user_presence_changed(
        &self,
        user_id: &UserOrResourceId,
        presence: &Presence,
    ) -> Result<()>;

    async fn handle_user_status_changed(
        &self,
        user_id: &UserId,
        user_activity: Option<&UserStatus>,
    ) -> Result<()>;

    async fn handle_avatar_changed(
        &self,
        user_id: &UserId,
        metadata: Option<&AvatarMetadata>,
    ) -> Result<()>;

    async fn handle_user_profile_changed(
        &self,
        user_id: &UserId,
        profile: Option<&UserProfile>,
    ) -> Result<()>;

    async fn reset_before_reconnect(&self) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
