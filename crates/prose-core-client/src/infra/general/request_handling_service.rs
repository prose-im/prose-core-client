// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use xmpp_parsers::version::VersionResult;

use prose_xmpp::mods;

use crate::domain::general::models::{Capabilities, Feature, Identity, SoftwareVersion};
use crate::domain::general::services::RequestHandlingService;
use crate::domain::shared::models::{RequestId, SenderId};
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RequestHandlingService for XMPPClient {
    async fn respond_to_ping(&self, to: &SenderId, id: &RequestId) -> anyhow::Result<()> {
        let ping = self.client.get_mod::<mods::Ping>();
        ping.send_pong(to.clone().into_inner(), id.as_ref()).await?;
        Ok(())
    }

    async fn respond_to_disco_info_query(
        &self,
        to: &SenderId,
        id: &RequestId,
        capabilities: &Capabilities,
    ) -> anyhow::Result<()> {
        let caps = self.client.get_mod::<mods::Caps>();
        caps.send_disco_info_query_response(
            to.clone().into_inner(),
            id.as_ref().to_string(),
            (&capabilities.clone()).into(),
        )
        .await?;
        Ok(())
    }

    async fn respond_to_entity_time_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        now: &DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .send_entity_time_response(now.clone().into(), to.clone().into_inner(), id.as_ref())
            .await?;
        Ok(())
    }

    async fn respond_to_software_version_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        version: &SoftwareVersion,
    ) -> anyhow::Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .send_software_version_response(
                version.clone().into(),
                to.clone().into_inner(),
                id.as_ref(),
            )
            .await?;
        Ok(())
    }

    async fn respond_to_last_activity_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        last_active_seconds_ago: u64,
    ) -> anyhow::Result<()> {
        let profile = self.client.get_mod::<mods::Profile>();
        profile
            .send_last_activity_response(
                last_active_seconds_ago,
                None,
                to.clone().into_inner(),
                id.as_ref(),
            )
            .await?;
        Ok(())
    }
}

impl Display for Feature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name(namespace) => {
                write!(f, "{}", namespace)
            }
            Self::Notify(namespace) => {
                write!(f, "{}+notify", namespace)
            }
        }
    }
}

impl From<&Capabilities> for xmpp_parsers::disco::DiscoInfoResult {
    fn from(value: &Capabilities) -> Self {
        xmpp_parsers::disco::DiscoInfoResult {
            node: None,
            identities: vec![(&value.identity).into()],
            features: value.features.iter().map(Into::into).collect(),
            extensions: vec![],
        }
    }
}

impl From<&Identity> for xmpp_parsers::disco::Identity {
    fn from(value: &Identity) -> Self {
        xmpp_parsers::disco::Identity {
            category: value.category.clone(),
            type_: value.kind.clone(),
            lang: Some(value.lang.clone()),
            name: Some(value.name.clone()),
        }
    }
}

impl From<&Feature> for xmpp_parsers::disco::Feature {
    fn from(value: &Feature) -> Self {
        xmpp_parsers::disco::Feature {
            var: value.to_string(),
        }
    }
}

impl From<SoftwareVersion> for VersionResult {
    fn from(value: SoftwareVersion) -> Self {
        VersionResult {
            name: value.name,
            version: value.version,
            os: value.os,
        }
    }
}
