// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;

use prose_xmpp::mods;
use prose_xmpp::stanza::VCard4;

use crate::domain::account::services::UserAccountService;
use crate::domain::general::models::Capabilities;
use crate::domain::shared::models::Availability;
use crate::domain::user_info::models::{AvatarImageId, AvatarMetadata, UserProfile, UserStatus};
use crate::dtos::OccupantId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserAccountService for XMPPClient {
    async fn set_avatar_metadata(&self, metadata: &AvatarMetadata) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_metadata(
                metadata.bytes,
                &metadata.checksum.as_ref().into(),
                &metadata.mime_type,
                metadata.width,
                metadata.height,
            )
            .await?;
        Ok(())
    }

    async fn set_avatar_image(
        &self,
        checksum: &AvatarImageId,
        base64_image_data: String,
    ) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_image(&checksum.as_ref().into(), base64_image_data)
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

    async fn set_profile(&self, user_profile: &UserProfile) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        let vcard = VCard4::from(user_profile.clone());
        profile.set_vcard(vcard.clone()).await?;
        profile.publish_vcard(vcard).await?;
        Ok(())
    }

    async fn delete_profile(&self) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile.unpublish_vcard().await?;
        profile.delete_vcard().await?;
        Ok(())
    }
}
