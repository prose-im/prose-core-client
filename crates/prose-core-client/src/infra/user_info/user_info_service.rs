// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jid::Jid;
use tracing::warn;
use xmpp_parsers::hashes::Sha1HexAttribute;

use prose_xmpp::mods::AvatarData;
use prose_xmpp::{mods, RequestError};

use crate::domain::shared::models::{AvatarId, BareEntityId, ParticipantIdRef, UserResourceId};
use crate::domain::user_info::models::{LastActivity, UserMetadata, UserProfile};
use crate::domain::user_info::services::UserInfoService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoService for XMPPClient {
    async fn load_avatar_image(
        &self,
        from: &BareEntityId,
        image_id: &AvatarId,
    ) -> Result<Option<AvatarData>, RequestError> {
        let profile = self.client.get_mod::<mods::Profile>();

        match profile
            .load_avatar_image(
                Jid::from(from.clone().into_inner()),
                &Sha1HexAttribute::from_str(&image_id.to_string()).map_err(|_| {
                    RequestError::Generic {
                        msg: "Invalid AvatarId".to_string(),
                    }
                })?,
            )
            .await
        {
            Ok(image) => Ok(image),
            Err(err) if err.is_forbidden_err() => {
                warn!("You don't have the rights to access the avatar of {}", from);
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn load_vcard_temp(
        &self,
        from: ParticipantIdRef<'_>,
    ) -> Result<Option<UserProfile>, RequestError> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .load_vcard_temp(from.to_owned())
            .await?
            .map(UserProfile::try_from)
            .transpose()
    }

    async fn load_user_metadata(
        &self,
        from: &UserResourceId,
        now: DateTime<Utc>,
    ) -> Result<Option<UserMetadata>, RequestError> {
        let profile = self.client.get_mod::<mods::Profile>();

        let entity_time = profile
            .load_entity_time(Jid::from(from.clone().into_inner()))
            .await?;
        let last_activity = profile
            .load_last_activity(Jid::from(from.clone().into_inner()))
            .await?;

        let metadata = UserMetadata {
            local_time: Some(entity_time),
            last_activity: Some(LastActivity {
                timestamp: now - Duration::seconds(last_activity.seconds as i64),
                status: last_activity.status.clone(),
            }),
        };

        Ok(Some(metadata))
    }
}
