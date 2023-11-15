use anyhow::Result;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{
    DynAccountSettingsRepository, DynAvatarRepository, DynContactsRepository, DynDraftsRepository,
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
    contacts_repo: DynContactsRepository,
    #[inject]
    drafts_repo: DynDraftsRepository,
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
        self.contacts_repo.clear_cache().await?;
        self.drafts_repo.clear_cache().await?;
        self.messages_repo.clear_cache().await?;
        self.sidebar_domain_service.clear_cache().await?;
        self.user_info_repo.clear_cache().await?;
        self.user_profile_repo.clear_cache().await?;
        Ok(())
    }
}
