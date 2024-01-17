// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::general::models::Capabilities;
use crate::domain::shared::models::Availability;
use crate::domain::user_info::models::{AvatarImageId, AvatarMetadata, UserStatus};
use crate::domain::user_profiles::models::UserProfile;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserAccountService: SendUnlessWasm + SyncUnlessWasm {
    async fn set_avatar_metadata(&self, metadata: &AvatarMetadata) -> Result<()>;
    async fn set_avatar_image(
        &self,
        checksum: &AvatarImageId,
        base64_image_data: String,
    ) -> Result<()>;

    async fn set_availability(
        &self,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<()>;
    async fn set_user_activity(&self, user_activity: Option<&UserStatus>) -> Result<()>;

    async fn set_profile(&self, profile: &UserProfile) -> Result<()>;
    async fn delete_profile(&self) -> Result<()>;
}
