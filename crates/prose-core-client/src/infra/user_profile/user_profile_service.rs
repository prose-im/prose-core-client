// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jid::Jid;
use tracing::warn;

use prose_xmpp::mods;

use crate::domain::shared::models::{UserId, UserResourceId};
use crate::domain::user_info::models::{LastActivity, UserMetadata, UserProfile};
use crate::domain::user_info::services::UserProfileService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserProfileService for XMPPClient {
    async fn load_profile(&self, from: &UserId) -> Result<Option<UserProfile>> {
        let profile = self.client.get_mod::<mods::Profile>();

        match profile.load_vcard(from.clone()).await {
            Ok(vcard) => Ok(vcard.map(TryInto::try_into).transpose()?),
            Err(err) if err.is_forbidden_err() => {
                warn!("You don't have the rights to access the vCard of {}", from);
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn load_user_metadata(
        &self,
        from: &UserResourceId,
        now: DateTime<Utc>,
    ) -> Result<Option<UserMetadata>> {
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
