// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::instrument;

use prose_xmpp::mods::Status;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{Availability, UserActivity};

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn set_availability(&self, availability: Availability) -> Result<()> {
        let status_mod = self.client.get_mod::<Status>();
        status_mod.send_presence(Some(availability.try_into()?), None, None)
    }

    #[instrument]
    pub async fn set_user_activity(&self, user_activity: Option<UserActivity>) -> Result<()> {
        let status_mod = self.client.get_mod::<Status>();
        status_mod
            .publish_activity(user_activity.map(Into::into).unwrap_or_default())
            .await
    }
}
