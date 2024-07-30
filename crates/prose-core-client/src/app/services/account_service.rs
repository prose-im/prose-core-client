// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use tracing::debug;

use prose_proc_macros::InjectDependencies;
use prose_xmpp::mods::AvatarData;

use crate::app::deps::*;
use crate::domain::account::services::UserProfileFormat;
use crate::domain::shared::models::{Availability, AvatarId, CachePolicy, ParticipantIdRef};
use crate::domain::user_info::models::{AvatarMetadata, UserProfile, UserStatus};
use crate::dtos::{AccountInfo, DeviceId, DeviceInfo, UserProfile as UserProfileDTO};
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
    encryption_domain_service: DynEncryptionDomainService,
    #[inject]
    user_account_service: DynUserAccountService,
    #[inject]
    user_info_domain_service: DynUserInfoDomainService,
}

impl AccountService {
    pub async fn account_info(&self) -> Result<AccountInfo> {
        let account = self.ctx.connected_account()?;
        let user_id = account.to_user_id();

        let user_info = self
            .user_info_domain_service
            .get_user_info(&user_id, CachePolicy::ReturnCacheDataElseLoad)
            .await?
            .unwrap_or_default();
        let name = user_info.display_name().unwrap_or_username(&user_id);
        let account_settings = self.account_settings_repo.get(&account).await?;

        Ok(AccountInfo {
            id: user_id,
            name,
            avatar: user_info.avatar,
            availability: account_settings.availability,
            status: user_info.status,
        })
    }

    pub async fn set_profile(&self, user_profile: UserProfileDTO) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let user_id = account.to_user_id();
        let user_profile = UserProfile::from(user_profile);

        let format = if self.ctx.server_features()?.vcard4 {
            UserProfileFormat::Vcard4
        } else {
            UserProfileFormat::VcardTemp
        };

        self.user_account_service
            .set_profile(user_profile.clone(), format)
            .await?;
        self.user_info_domain_service
            .handle_user_profile_changed(&user_id, Some(user_profile))
            .await?;

        Ok(())
    }

    pub async fn set_availability(&self, availability: Availability) -> Result<()> {
        let account = self.ctx.connected_account()?;

        self.user_account_service
            .set_availability(None, &self.ctx.capabilities, availability)
            .await?;

        for room in self.connected_rooms_repo.get_all(&account) {
            let Some(occupant_id) = room.occupant_id() else {
                continue;
            };
            self.user_account_service
                .set_availability(Some(occupant_id), &self.ctx.capabilities, availability)
                .await?
        }

        self.account_settings_repo
            .update(
                &account,
                Box::new(move |settings| settings.availability = availability),
            )
            .await?;

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::AccountInfoChanged);

        Ok(())
    }

    pub async fn set_user_activity(&self, user_activity: Option<UserStatus>) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let user_id = account.to_user_id();

        self.user_account_service
            .set_user_activity(user_activity.as_ref())
            .await?;
        self.user_info_domain_service
            .handle_user_status_changed(&user_id, user_activity)
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
        let account = self.ctx.connected_account()?;
        let user_id = account.to_user_id();
        let image_data_len = image_data.as_ref().len();
        let image_data = AvatarData::Data(image_data.as_ref().to_vec().into_boxed_slice());

        let metadata = AvatarMetadata {
            bytes: image_data_len,
            mime_type: mime_type.as_ref().to_string(),
            checksum: AvatarId::from_str_unchecked(image_data.generate_sha1_checksum()?.as_ref()),
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

        debug!("Caching image locally…");
        self.avatar_repo
            .set(
                &account,
                ParticipantIdRef::User(&user_id),
                &metadata.clone().into_info(),
                &image_data,
            )
            .await?;

        debug!("Caching avatar metadata");
        self.user_info_domain_service
            .handle_avatar_changed(&user_id, Some(metadata))
            .await?;

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn set_avatar_from_url(&self, image_path: &std::path::Path) -> Result<()> {
        use crate::infra::constants::MAX_IMAGE_DIMENSIONS;
        use image::{codecs::jpeg::JpegEncoder, GenericImageView};
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
        let encoder = JpegEncoder::new_with_quality(&mut image_data, 94);
        img.write_with_encoder(encoder)?;

        self.set_avatar(
            image_data,
            Some(img.dimensions().0),
            Some(img.dimensions().1),
            "image/jpeg",
        )
        .await
    }

    pub async fn load_device_infos(&self) -> Result<Vec<DeviceInfo>> {
        self.encryption_domain_service
            .load_device_infos(&self.ctx.connected_id()?.into_user_id())
            .await
    }

    pub async fn delete_device(&self, device_id: &DeviceId) -> Result<()> {
        self.encryption_domain_service
            .delete_device(device_id)
            .await
    }

    pub async fn disable_omemo(&self) -> Result<()> {
        self.encryption_domain_service.disable_omemo().await
    }
}
