use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAccountSettingsRepository, DynAppContext, DynAvatarRepository, DynBlockListDomainService,
    DynContactListDomainService, DynDraftsRepository, DynEncryptionDomainService,
    DynLocalRoomSettingsRepository, DynMessagesRepository, DynSidebarDomainService,
    DynUserInfoDomainService,
};

#[derive(InjectDependencies)]
pub struct CacheService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    account_settings_repo: DynAccountSettingsRepository,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    block_list_domain_service: DynBlockListDomainService,
    #[inject]
    contact_list_domain_service: DynContactListDomainService,
    #[inject]
    drafts_repo: DynDraftsRepository,
    #[inject]
    encryption_domain_service: DynEncryptionDomainService,
    #[inject]
    local_room_settings_repo: DynLocalRoomSettingsRepository,
    #[inject]
    messages_repo: DynMessagesRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
}

impl CacheService {
    pub async fn clear_cache(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;

        self.account_settings_repo.clear_cache(&account).await?;
        self.avatar_repo.clear_cache(&account).await?;
        self.drafts_repo.clear_cache(&account).await?;
        self.local_room_settings_repo.clear_cache(&account).await?;
        self.messages_repo.clear_cache(&account).await?;

        self.block_list_domain_service.clear_cache().await?;
        self.contact_list_domain_service.clear_cache().await?;
        self.encryption_domain_service.clear_cache().await?;
        self.sidebar_domain_service.clear_cache().await?;
        self.user_info_domain_service.clear_cache().await?;

        Ok(())
    }
}
