// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::debug;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::AvatarData;

use crate::app::deps::*;
use crate::domain::shared::models::Availability;
use crate::domain::user_info::models::{AvatarMetadata, UserStatus};
use crate::domain::user_profiles::models::UserProfile;
use crate::dtos::AccountInfo;
use crate::ClientEvent;

#[derive(InjectDependencies)]
pub struct AccountService {
    #[inject]
    account_settings_repo: DynAccountSettingsRepository,
    #[inject]
    avatar_repo: DynAvatarRepository,
    #[inject]
    connected_rooms_repo: DynConnectedRoomsReadOnlyRepository,
    #[inject]
    ctx: DynAppContext,
    #[inject]
    client_event_dispatcher: DynClientEventDispatcher,
    #[inject]
    user_account_service: DynUserAccountService,
    #[inject]
    user_info_repo: DynUserInfoRepository,
    #[inject]
    user_profile_repo: DynUserProfileRepository,
}

impl AccountService {
    pub async fn account_info(&self) -> Result<AccountInfo> {
        let user_id = self.ctx.connected_id()?.into_user_id();
        let user_info = self.user_info_repo.get_user_info(&user_id).await?;
        let account_settings = self.account_settings_repo.get(&user_id).await?;
        let name = self
            .user_profile_repo
            .get_display_name(&user_id)
            .await?
            .unwrap_or_else(|| user_id.formatted_username());

        Ok(AccountInfo {
            id: user_id,
            name,
            availability: account_settings.availability,
            status: user_info.and_then(|info| info.activity),
        })
    }

    pub async fn set_profile(&self, user_profile: &UserProfile) -> Result<()> {
        self.user_account_service.set_profile(&user_profile).await?;
        self.user_profile_repo
            .set(&self.ctx.connected_id()?.to_user_id(), user_profile)
            .await?;
        Ok(())
    }

    pub async fn delete_profile(&self) -> Result<()> {
        self.user_account_service.delete_profile().await?;
        self.user_profile_repo
            .delete(&self.ctx.connected_id()?.to_user_id())
            .await?;
        Ok(())
    }

    pub async fn set_availability(&self, availability: Availability) -> Result<()> {
        self.user_account_service
            .set_availability(None, &self.ctx.capabilities, availability)
            .await?;

        for room in self.connected_rooms_repo.get_all() {
            let Some(occupant_id) = room.occupant_id() else {
                continue;
            };
            self.user_account_service
                .set_availability(Some(occupant_id), &self.ctx.capabilities, availability)
                .await?
        }

        self.account_settings_repo
            .update(
                &self.ctx.connected_id()?.to_user_id(),
                Box::new(move |settings| settings.availability = availability),
            )
            .await?;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::AccountInfoChanged);

        Ok(())
    }

    pub async fn set_user_activity(&self, user_activity: Option<UserStatus>) -> Result<()> {
        self.user_account_service
            .set_user_activity(user_activity.as_ref())
            .await?;
        self.user_info_repo
            .set_user_activity(
                &self.ctx.connected_id()?.to_user_id(),
                user_activity.as_ref(),
            )
            .await?;
        Ok(())
    }

    pub async fn set_avatar(
        &self,
        image_data: impl AsRef<[u8]>,
        width: Option<u32>,
        height: Option<u32>,
        mime_type: impl AsRef<str>,
    ) -> Result<()> {
        let jid = self.ctx.connected_id()?.to_user_id();
        let image_data_len = image_data.as_ref().len();
        let image_data = AvatarData::Data(image_data.as_ref().to_vec());

        let metadata = AvatarMetadata {
            bytes: image_data_len,
            mime_type: mime_type.as_ref().to_string(),
            checksum: image_data.generate_sha1_checksum()?.as_ref().into(),
            width,
            height,
            url: None,
        };

        debug!("Uploading avatar…");
        self.user_account_service
            .set_avatar_image(&metadata.checksum, image_data.base64().to_string())
            .await?;

        debug!("Uploading avatar metadata…");
        self.user_account_service
            .set_avatar_metadata(&metadata)
            .await?;

        debug!("Caching avatar metadata");
        self.user_info_repo
            .set_avatar_metadata(&jid, &metadata)
            .await?;

        debug!("Caching image locally…");
        self.avatar_repo
            .set(&jid, &metadata.into_info(), &image_data)
            .await?;

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn set_avatar_from_url(&self, image_path: &std::path::Path) -> Result<()> {
        use crate::infra::constants::{
            IMAGE_OUTPUT_FORMAT, IMAGE_OUTPUT_MIME_TYPE, MAX_IMAGE_DIMENSIONS,
        };
        use image::GenericImageView;
        use std::io::Cursor;
        use std::time::Instant;

        let now = Instant::now();
        debug!("Opening & resizing image at {:?}…", image_path);

        let img =
            image::open(image_path)?.thumbnail(MAX_IMAGE_DIMENSIONS.0, MAX_IMAGE_DIMENSIONS.1);
        debug!(
            "Opening image & resizing finished after {:.2?}",
            now.elapsed()
        );

        let mut image_data = Vec::new();
        img.write_to(&mut Cursor::new(&mut image_data), IMAGE_OUTPUT_FORMAT)?;

        self.set_avatar(
            image_data,
            Some(img.dimensions().0),
            Some(img.dimensions().1),
            IMAGE_OUTPUT_MIME_TYPE,
        )
        .await
    }
}
