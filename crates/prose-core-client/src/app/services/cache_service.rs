use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAccountSettingsRepository, DynAvatarRepository, DynBlockListDomainService,
    DynContactListDomainService, DynDraftsRepository, DynEncryptionDomainService,
    DynMessagesRepository, DynSidebarDomainService, DynUserInfoRepository,
    DynUserProfileRepository,
};

#[derive(InjectDependencies)]
pub struct CacheService {
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
    messages_repo: DynMessagesRepository,
    #[inject]
    sidebar_domain_service: DynSidebarDomainService,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl CacheService {
    pub async fn clear_cache(&self) -> Result<()> {
        self.account_settings_repo.clear_cache().await?;
        self.avatar_repo.clear_cache().await?;
        self.block_list_domain_service.clear_cache().await?;
        self.contact_list_domain_service.clear_cache().await?;
        self.drafts_repo.clear_cache().await?;
        self.encryption_domain_service.clear_cache().await?;
        self.messages_repo.clear_cache().await?;
        self.sidebar_domain_service.clear_cache().await?;
        self.user_info_repo.clear_cache().await?;
        self.user_profile_repo.clear_cache().await?;
        Ok(())
    }
}
