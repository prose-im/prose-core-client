// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::contacts::models::{Contact, PresenceSubscription};
use crate::domain::shared::models::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ContactListDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_contacts(&self) -> Result<Vec<Contact>>;
    async fn add_contact(&self, user_id: &UserId) -> Result<()>;
    async fn remove_contact(&self, user_id: &UserId) -> Result<()>;

    /// Requests a presence subscription from `from`. Note that happens automatically when you
    /// call `add_contact`. This method can be useful though when our user needs to re-request
    /// the presence subscription in case the contact hasn't reacted in a while.
    async fn request_presence_sub(&self, from: &UserId) -> Result<()>;

    async fn load_presence_sub_requests(&self) -> Result<Vec<UserId>>;
    async fn approve_presence_sub_request(&self, from: &UserId) -> Result<()>;
    async fn deny_presence_sub_request(&self, from: &UserId) -> Result<()>;

    async fn handle_updated_contact(
        &self,
        user_id: &UserId,
        subscription: PresenceSubscription,
    ) -> Result<()>;
    async fn handle_removed_contact(&self, user_id: &UserId) -> Result<()>;
    async fn handle_presence_sub_request(&self, from: &UserId) -> Result<()>;

    async fn clear_cache(&self) -> Result<()>;
}
