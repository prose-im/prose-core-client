// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;

use prose_xmpp::mods;

use crate::domain::contacts::services::BlockListService;
use crate::dtos::UserId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl BlockListService for XMPPClient {
    async fn load_block_list(&self) -> Result<Vec<UserId>> {
        let block_list_mod = self.client.get_mod::<mods::BlockList>();
        Ok(block_list_mod
            .load_block_list()
            .await?
            .into_iter()
            .map(|jid| UserId::from(jid.into_bare()))
            .collect())
    }

    async fn block_user(&self, user_id: &UserId) -> Result<()> {
        let block_list_mod = self.client.get_mod::<mods::BlockList>();
        block_list_mod
            .block_user(&Jid::from(user_id.clone().into_inner()))
            .await?;
        Ok(())
    }

    async fn unblock_user(&self, user_id: &UserId) -> Result<()> {
        let block_list_mod = self.client.get_mod::<mods::BlockList>();
        block_list_mod
            .unblock_user(&Jid::from(user_id.clone().into_inner()))
            .await?;
        Ok(())
    }

    async fn clear_block_list(&self) -> Result<()> {
        let block_list_mod = self.client.get_mod::<mods::BlockList>();
        block_list_mod.clear_block_list().await?;
        Ok(())
    }
}
