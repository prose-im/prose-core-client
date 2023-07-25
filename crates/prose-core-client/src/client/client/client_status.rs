use anyhow::Result;
use tracing::instrument;

use prose_xmpp::mods::Status;

use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use crate::types::{Availability, UserActivity};

use super::Client;

impl<D: DataCache, A: AvatarCache> Client<D, A> {
    #[instrument]
    pub async fn set_availability(
        &self,
        availability: Availability,
        status: Option<&str>,
    ) -> Result<()> {
        let status_mod = self.client.get_mod::<Status>();
        status_mod.send_presence(Some(availability.try_into()?), status)
    }

    #[instrument]
    pub async fn set_user_activity(&self, user_activity: Option<UserActivity>) -> Result<()> {
        let status_mod = self.client.get_mod::<Status>();
        status_mod
            .publish_activity(user_activity.map(Into::into).unwrap_or_default())
            .await
    }
}
