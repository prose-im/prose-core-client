// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{DeviceId, Session};
use crate::domain::shared::models::UserId;
use crate::dtos::{IdentityKey, SessionData};

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait SessionRepository: SendUnlessWasm + SyncUnlessWasm {
    async fn get_session(&self, user_id: &UserId, device_id: &DeviceId) -> Result<Option<Session>>;
    async fn get_all_sessions(&self, user_id: &UserId) -> Result<Vec<Session>>;

    async fn put_session_data(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        data: SessionData,
    ) -> Result<()>;

    /// Record an identity into the store. The identity is then considered "undecided".
    ///
    /// The return value represents whether an existing identity was replaced (`Ok(true)`). If it is
    /// new or hasn't changed, the return value should be `Ok(false)`.
    async fn put_identity(
        &self,
        user_id: &UserId,
        device_id: &DeviceId,
        identity: IdentityKey,
    ) -> Result<bool>;

    /// Marks all sessions not included in `device_ids` as inactive.
    async fn put_active_devices(&self, user_id: &UserId, device_ids: &[DeviceId]) -> Result<()>;

    async fn clear_cache(&self) -> Result<()>;
}
