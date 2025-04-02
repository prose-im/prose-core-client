// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;

use prose_xmpp::mods;

use crate::domain::account::services::{UserAccountService, UserProfileFormat};
use crate::domain::general::models::Capabilities;
use crate::domain::shared::models::{Availability, AvatarId, AvatarMetadata, OccupantId, UserId};
use crate::domain::shared::utils::ContactNameBuilder;
use crate::domain::user_info::models::{UserProfile, UserStatus};
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserAccountService for XMPPClient {
    async fn change_password(&self, new_password: &str) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile.change_password(new_password).await?;
        Ok(())
    }

    async fn set_avatar_metadata(&self, metadata: &AvatarMetadata) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_metadata(
                metadata.bytes,
                &metadata.checksum.to_string().into(),
                &metadata.mime_type,
                metadata.width,
                metadata.height,
            )
            .await?;
        Ok(())
    }

    async fn set_avatar_image(&self, checksum: &AvatarId, base64_image_data: String) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_image(&checksum.to_string().into(), base64_image_data)
            .await?;
        Ok(())
    }

    async fn set_availability(
        &self,
        room_id: Option<OccupantId>,
        capabilities: &Capabilities,
        availability: Availability,
    ) -> Result<()> {
        let status_mod = self.client.get_mod::<mods::Status>();
        status_mod.send_presence(
            room_id.map(|id| Jid::from(id.into_inner())),
            Some(availability.try_into()?),
            None,
            Some(capabilities.into()),
            None,
        )
    }

    async fn set_user_activity(&self, user_activity: Option<&UserStatus>) -> Result<()> {
        let status_mod = self.client.get_mod::<mods::Status>();
        status_mod
            .publish_activity(user_activity.cloned().map(Into::into).unwrap_or_default())
            .await
    }

    async fn set_profile(
        &self,
        user_profile: UserProfile,
        format: UserProfileFormat,
    ) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();

        let nickname = ContactNameBuilder::new()
            .or_nickname(user_profile.nickname.as_ref())
            .or_firstname_lastname(
                user_profile.first_name.as_ref(),
                user_profile.last_name.as_ref(),
            )
            .or_username(
                self.client
                    .connected_jid()
                    .map(|jid| UserId::from(jid.into_bare()))
                    .as_ref(),
            )
            .build();

        match format {
            UserProfileFormat::Vcard4 => {
                profile.publish_vcard4(user_profile.into(), None).await?;
            }
            UserProfileFormat::VcardTemp => {
                profile.publish_vcard_temp(user_profile.into()).await?;
            }
        }

        profile.publish_nickname(nickname).await?;

        Ok(())
    }

    async fn delete_profile(&self) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile.unpublish_vcard().await?;
        profile.delete_vcard().await?;
        Ok(())
    }
}
