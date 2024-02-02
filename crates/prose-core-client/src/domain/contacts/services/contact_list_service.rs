// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::contacts::models::Contact;
use crate::dtos::UserId;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait ContactListService: SendUnlessWasm + SyncUnlessWasm {
    async fn load_contacts(&self) -> Result<Vec<Contact>>;
    async fn add_contact(&self, user_id: &UserId) -> Result<()>;
    async fn remove_contact(&self, user_id: &UserId) -> Result<()>;

    async fn subscribe_to_presence(&self, user_id: &UserId) -> Result<()>;
    async fn unsubscribe_from_presence(&self, user_id: &UserId) -> Result<()>;
    async fn revoke_presence_subscription(&self, user_id: &UserId) -> Result<()>;
    async fn preapprove_subscription_request(&self, user_id: &UserId) -> Result<()>;

    async fn approve_presence_sub_request(&self, to: &UserId) -> Result<()>;
    async fn deny_presence_sub_request(&self, to: &UserId) -> Result<()>;
}
