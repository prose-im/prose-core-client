// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::general::models::Capabilities;
use crate::domain::shared::models::{Availability, AvatarId};
use crate::domain::user_info::models::{AvatarMetadata, UserProfile, UserStatus};
use crate::dtos::OccupantId;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum UserProfileFormat {
    Vcard4,
    VcardTemp,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait UserAccountService: SendUnlessWasm + SyncUnlessWasm {
    async fn change_password(&self, new_password: &str) -> Result<()>;
    async fn set_avatar_metadata(&self, metadata: &AvatarMetadata) -> Result<()>;
    async fn set_avatar_image(&self, checksum: &AvatarId, base64_image_data: String) -> Result<()>;

    async fn set_availability(
        &self,
        occupant_id: Option<OccupantId>,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<()>;

    async fn set_user_activity(&self, user_activity: Option<&UserStatus>) -> Result<()>;

    async fn set_profile(&self, profile: UserProfile, format: UserProfileFormat) -> Result<()>;
    async fn delete_profile(&self) -> Result<()>;
}
