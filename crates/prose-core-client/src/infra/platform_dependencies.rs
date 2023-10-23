// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use prose_store::prelude::*;

use crate::app::deps::AppServiceDependencies;
use crate::infra::avatars::AvatarCache;
use crate::infra::messaging::{DraftsRecord, MessagesRecord};
use crate::infra::settings::AccountSettingsRecord;
use crate::infra::user_info::UserInfoRecord;
use crate::infra::user_profile::UserProfileRecord;
use crate::infra::xmpp::XMPPClient;

pub struct PlatformDependencies {
    pub app_service: AppServiceDependencies,
    pub store: Store<PlatformDriver>,
    pub xmpp: Arc<XMPPClient>,
    pub avatar_cache: Box<dyn AvatarCache>,
}

pub async fn open_store<D: Driver>(driver: D) -> Result<Store<D>, D::Error> {
    Store::open(driver, 10, |event| {
        let tx = &event.tx;

        create_collection::<D, AccountSettingsRecord>(&tx)?;
        create_collection::<D, DraftsRecord>(&tx)?;
        create_collection::<D, MessagesRecord>(&tx)?;
        create_collection::<D, UserInfoRecord>(&tx)?;
        create_collection::<D, UserProfileRecord>(&tx)?;

        Ok(())
    })
    .await
}

fn create_collection<D: Driver, E: Entity>(tx: &D::UpgradeTransaction<'_>) -> Result<(), D::Error> {
    let collection = tx.create_collection(E::collection())?;
    for idx_spec in E::indexes() {
        collection.add_index(idx_spec)?;
    }
    Ok(())
}

// impl From<PlatformDependencies> for AppDependencies {
//     fn from(d: PlatformDependencies) -> Self {
//         Self {
//             account_settings_repo: Arc::new(infra::settings::AccountSettingsRepository {
//                 store: d.store.clone(),
//             }),
//             app_service: d.app_service,
//             avatar_repo: d.avatar_repo,
//             bookmarks_repo: (),
//             contacts_repo: Arc::new(()),
//             contacts_service: Arc::new(()),
//             ctx: Arc::new(AppContext {}),
//             drafts_repo: DraftsRepository {},
//             request_handling_service: Arc::new(()),
//             room_factory: (),
//             room_management_service: Arc::new(()),
//             room_message_archive_service: Arc::new(()),
//             room_messages_repo: MessagesRepository {},
//             room_messaging_service: Arc::new(()),
//             room_participation_service: Arc::new(()),
//             room_topic_service: Arc::new(()),
//             user_account_service: Arc::new(()),
//             user_info_repo: Arc::new(()),
//             user_info_service: Arc::new(()),
//             user_profile_repo: Arc::new(()),
//             user_profile_service: Arc::new(()),
//         }
//     }
// }
