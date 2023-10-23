// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use xmpp_parsers::presence;

use prose_xmpp::mods;
use prose_xmpp::stanza::{avatar, VCard4};

use crate::domain::account::services::UserAccountService;
use crate::domain::general::models::Capabilities;
use crate::domain::shared::models::Availability;
use crate::domain::user_info::models::{AvatarMetadata, UserActivity};
use crate::domain::user_profiles::models::UserProfile;
use crate::infra::xmpp::XMPPClient;

#[async_trait]
impl UserAccountService for XMPPClient {
    async fn set_avatar_metadata(&self, metadata: &AvatarMetadata) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_metadata(
                metadata.bytes,
                &metadata.checksum,
                &metadata.mime_type,
                metadata.width,
                metadata.height,
            )
            .await?;
        Ok(())
    }

    async fn set_avatar_image(
        &self,
        checksum: &avatar::ImageId,
        base64_image_data: String,
    ) -> Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .set_avatar_image(checksum, base64_image_data)
            .await?;
        Ok(())
    }

    async fn set_availability(
        &self,
        capabilities: &Capabilities,
        availability: &Availability,
    ) -> Result<()> {
        let status_mod = self.client.get_mod::<mods::Status>();
        status_mod.send_presence(
            Some(availability.clone().try_into()?),
            None,
            Some(capabilities.into()),
        )
    }

    async fn set_user_activity(&self, user_activity: Option<&UserActivity>) -> Result<()> {
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

impl From<(Option<presence::Type>, Option<presence::Show>)> for Availability {
    fn from(value: (Option<presence::Type>, Option<presence::Show>)) -> Self {
        // https://datatracker.ietf.org/doc/html/rfc6121#section-4.7.1
        match (value.0, value.1) {
            // The absence of a 'type' attribute signals that the relevant entity is
            // available for communication (see Section 4.2 and Section 4.4).
            (None, None) => Availability::Available,
            (None, Some(presence::Show::Away)) => Availability::Away,
            (None, Some(presence::Show::Chat)) => Availability::Available,
            (None, Some(presence::Show::Dnd)) => Availability::DoNotDisturb,
            (None, Some(presence::Show::Xa)) => Availability::Away,
            (Some(_), _) => Availability::Unavailable,
        }
    }
}

impl TryFrom<Availability> for presence::Show {
    type Error = anyhow::Error;

    fn try_from(value: Availability) -> Result<Self, Self::Error> {
        match value {
            Availability::Available => Ok(presence::Show::Chat),
            Availability::Unavailable => Err(anyhow::format_err!(
                "You cannot set yourself to Unavailable. Choose 'Away' instead."
            )),
            Availability::DoNotDisturb => Ok(presence::Show::Dnd),
            Availability::Away => Ok(presence::Show::Away),
        }
    }
}
