// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::general::models::{Capabilities, SoftwareVersion};
use crate::domain::shared::models::{RequestId, SenderId};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait RequestHandlingService: SendUnlessWasm + SyncUnlessWasm {
    async fn respond_to_ping(&self, to: &SenderId, id: &RequestId) -> Result<()>;

    async fn respond_to_disco_info_query(
        &self,
        to: &SenderId,
        id: &RequestId,
        capabilities: &Capabilities,
    ) -> Result<()>;

    async fn respond_to_entity_time_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        now: &DateTime<Utc>,
    ) -> Result<()>;

    async fn respond_to_software_version_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        version: &SoftwareVersion,
    ) -> Result<()>;

    async fn respond_to_last_activity_request(
        &self,
        to: &SenderId,
        id: &RequestId,
        last_active_seconds_ago: u64,
    ) -> Result<()>;
}
