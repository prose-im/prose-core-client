// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use futures::future::join_all;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynBlockListDomainService, DynUserProfileRepository};
use crate::domain::shared::models::AccountId;
use crate::domain::shared::utils::build_contact_name;
use crate::dtos::{UserBasicInfo, UserId};

#[derive(InjectDependencies)]
pub struct BlockListService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    block_list_domain_service: DynBlockListDomainService,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl BlockListService {
    pub async fn load_block_list(&self) -> Result<Vec<UserBasicInfo>> {
        let account = self.ctx.connected_account()?;
        let blocked_user_ids = self.block_list_domain_service.load_block_list().await?;
        let blocked_users = join_all(
            blocked_user_ids
                .into_iter()
                .map(|id| self.enrich_blocked_user(&account, id)),
        )
        .await;

        Ok(blocked_users)
    }

    pub async fn block_user(&self, user_id: &UserId) -> Result<()> {
        self.block_list_domain_service.block_user(user_id).await?;
        Ok(())
    }

    pub async fn unblock_user(&self, user_id: &UserId) -> Result<()> {
        self.block_list_domain_service.unblock_user(user_id).await?;
        Ok(())
    }

    pub async fn clear_block_list(&self) -> Result<()> {
        self.block_list_domain_service.clear_block_list().await?;
        Ok(())
    }
}

impl BlockListService {
    async fn enrich_blocked_user(&self, account: &AccountId, user_id: UserId) -> UserBasicInfo {
        let profile = self
            .user_profile_repo
            .get(&account, &user_id)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        let name = build_contact_name(&user_id, &profile);
        UserBasicInfo {
            id: user_id.into(),
            name,
        }
    }
}
