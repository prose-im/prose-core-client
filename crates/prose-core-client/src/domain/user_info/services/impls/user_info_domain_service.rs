// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::borrow::{Borrow, Cow};
use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use parking_lot::RwLock;
use tracing::{error, warn};

use prose_proc_macros::DependenciesStruct;
use prose_xmpp::mods::AvatarData;

use crate::app::deps::{
    DynAppContext, DynAvatarRepository, DynBlockListRepository, DynClientEventDispatcher,
    DynTimeProvider, DynUserInfoRepository, DynUserInfoService, DynUserProfileRepository,
};
use crate::domain::contacts::models::Contact;
use crate::domain::shared::models::{
    CachePolicy, ConnectionState, ParticipantIdRef, UserId, UserOrResourceId,
};
use crate::domain::user_info::models::{
    Avatar, AvatarInfo, AvatarMetadata, AvatarSource, Image, PlatformImage, Presence, ProfileName,
    UserInfo, UserMetadata, UserProfile, UserStatus,
};
use crate::domain::user_info::services::UserInfoDomainService as UserInfoDomainServiceTrait;
use crate::dtos::ParticipantId;
use crate::ClientEvent;

#[derive(DependenciesStruct)]
pub struct UserInfoDomainService {
    avatar_repo: DynAvatarRepository,
    block_list_repo: DynBlockListRepository,
    client_event_dispatcher: DynClientEventDispatcher,
    ctx: DynAppContext,
    time_provider: DynTimeProvider,
    user_info_repo: DynUserInfoRepository,
    user_info_service: DynUserInfoService,
    user_profile_repo: DynUserProfileRepository,

    requested_vcards: RwLock<HashSet<Jid>>,
    requested_avatars: RwLock<HashSet<Jid>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoDomainServiceTrait for UserInfoDomainService {
    async fn get_user_info(
        &self,
        user_id: &UserId,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserInfo>> {
        let account = self.ctx.connected_account()?;
        let info = self.user_info_repo.get(&account, user_id).await?;

        let should_load_vcard = info.is_none()
            || info.as_ref().map(|info| {
                // We check here if either the vCard was loaded already, or if we have enough data
                // to construct the name so that we don't need the vCard at all.
                info.name.vcard.is_none()
                    && info.name.nickname.is_none()
                    && info.name.presence.is_none()
            }) == Some(true);

        match cache_policy {
            CachePolicy::ReturnCacheDataDontLoad => Ok(info),
            CachePolicy::ReturnCacheDataElseLoad if !should_load_vcard => Ok(info),
            CachePolicy::ReturnCacheDataElseLoad => {
                self.load_user_profile_and_update_user_info(
                    user_id.into(),
                    CachePolicy::ReturnCacheDataElseLoad,
                )
                .await?;
                self.user_info_repo.get(&account, user_id).await
            }
        }
    }

    async fn get_user_profile(
        &self,
        user_id: &UserId,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserProfile>> {
        self.load_user_profile_and_update_user_info(user_id.into(), cache_policy)
            .await
    }

    async fn get_user_metadata(&self, user_id: &UserId) -> Result<Option<UserMetadata>> {
        let account = self.ctx.connected_account()?;

        if self.block_list_repo.contains(&account, user_id).await? {
            return Ok(None);
        }

        let Some(resource_id) = self
            .user_info_repo
            .resolve_user_id(&self.ctx.connected_account()?, user_id)
        else {
            return Ok(None);
        };

        let result = self
            .user_info_service
            .load_user_metadata(&resource_id, self.time_provider.now())
            .await;

        match result {
            Ok(metadata) => Ok(metadata),
            Err(err) if err.is_forbidden_err() => {
                warn!("You don't have the rights to access the metadata of {user_id}");
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn load_avatar_image(&self, avatar: &Avatar) -> Result<Option<PlatformImage>> {
        let account = self.ctx.connected_account()?;

        // If we have a real id for the requested avatar, let's use that one. This fixes at least
        // an issue where a contact published a vCard avatar in a MUC room but only had the avatar
        // set on their PEP node and not on the vCard itself.
        if let Some(real_id) = avatar.real_id() {
            if let Ok(Some(info)) = self
                .get_user_info(&real_id, CachePolicy::ReturnCacheDataDontLoad)
                .await
            {
                if let Some(avatar) = info.avatar {
                    if let Ok(Some(image)) = self.load_avatar_image(&avatar).await {
                        return Ok(Some(image));
                    }
                }
            }
        }

        if let Some(image) = self
            .avatar_repo
            .get(&account, avatar.owner(), &avatar.id)
            .await?
        {
            return Ok(Some(image));
        }

        if self
            .requested_avatars
            .read()
            .contains(avatar.owner().borrow())
        {
            return Ok(None);
        }

        let avatar = match &avatar.source {
            // If we have a vCard avatar with a UserId, we check if we happen to have
            // a PEP avatar for the same id…
            AvatarSource::Vcard {
                owner: ParticipantId::User(user_id),
                ..
            } => {
                if let Some(avatar) = self
                    .user_info_repo
                    .get(&account, user_id)
                    .await?
                    .and_then(|info| info.avatar)
                    .and_then(|avatar| avatar.is_pep().then_some(avatar))
                {
                    Cow::Owned(avatar)
                } else {
                    Cow::Borrowed(avatar)
                }
            }
            _ => Cow::Borrowed(avatar),
        };

        let result: Result<Option<(AvatarData, String)>> = match &avatar.source {
            AvatarSource::Pep { owner, mime_type } => self
                .user_info_service
                .load_avatar_image(owner, &avatar.id)
                .await
                .map_err(Into::into)
                .map(|data| data.map(|data| (data, mime_type.clone()))),
            AvatarSource::Vcard { owner, .. } => self
                .load_user_profile_and_update_user_info(
                    owner.to_ref(),
                    CachePolicy::ReturnCacheDataElseLoad,
                )
                .await
                .map(|vcard| {
                    vcard.and_then(|vcard| match vcard.photo {
                        None => None,
                        Some(Image::Binary { media_type, data }) => {
                            Some((AvatarData::Data(data), media_type))
                        }
                        Some(Image::External(url)) => {
                            error!(
                                "Encountered avatar URL {url:?} for {} which is not yet supported.",
                                avatar.owner()
                            );
                            None
                        }
                    })
                }),
        };

        self.requested_avatars
            .write()
            .insert(avatar.owner().as_ref().clone());

        let (data, mime_type) = match result {
            Ok(Some((data, mime_type))) => (data, mime_type),
            Ok(None) => return Ok(None),
            Err(err) => return Err(err),
        };

        self.avatar_repo
            .set(
                &account,
                avatar.owner(),
                &AvatarInfo {
                    checksum: avatar.id.clone(),
                    mime_type: mime_type.clone(),
                },
                &data,
            )
            .await?;

        self.avatar_repo
            .get(&account, avatar.owner(), &avatar.id)
            .await
    }

    async fn handle_user_presence_changed(
        &self,
        user_id: &UserOrResourceId,
        presence: Presence,
    ) -> Result<()> {
        self.user_info_repo
            .set_user_presence(&self.ctx.connected_account()?, user_id, &presence)
            .await?;

        let user_id = match user_id {
            UserOrResourceId::User(id) => id,
            UserOrResourceId::UserResource(id) => &id.to_user_id(),
        };

        self.update_user_info(user_id, move |info| {
            info.availability = presence.availability;
            info.caps = presence.caps;
            info.client = presence.client;
            info.name.presence = presence.nickname;

            // If we have a PEP avatar already, we'll keep it…
            if !info
                .avatar
                .as_ref()
                .map(|avatar| avatar.is_pep())
                .unwrap_or(false)
            {
                info.avatar = presence.avatar;
            }
        })
        .await?;

        Ok(())
    }

    async fn handle_user_status_changed(
        &self,
        user_id: &UserId,
        status: Option<UserStatus>,
    ) -> Result<()> {
        self.update_user_info(user_id, |info| info.status = status)
            .await
    }

    async fn handle_avatar_changed(
        &self,
        user_id: &UserId,
        metadata: Option<AvatarMetadata>,
    ) -> Result<()> {
        let avatar = metadata.map(|metadata| Avatar {
            id: metadata.checksum,
            source: AvatarSource::Pep {
                owner: user_id.clone(),
                mime_type: metadata.mime_type,
            },
        });

        self.update_user_info(user_id, |info| info.avatar = avatar)
            .await
    }

    async fn handle_user_profile_changed(
        &self,
        user_id: &UserId,
        profile: Option<UserProfile>,
    ) -> Result<()> {
        let account = self.ctx.connected_account()?;

        self.user_profile_repo
            .set(&account, user_id.into(), profile.as_ref())
            .await?;

        self.update_user_info(user_id, |info| {
            info.name.vcard = profile.map(|profile| ProfileName {
                first_name: profile.first_name,
                last_name: profile.last_name,
                nickname: profile.nickname,
            })
        })
        .await
    }

    async fn handle_nickname_changed(
        &self,
        user_id: &UserId,
        nickname: Option<String>,
    ) -> Result<()> {
        self.update_user_info(user_id, |info| info.name.nickname = nickname)
            .await
    }

    async fn handle_contacts_changed(&self, contacts: Vec<Contact>) -> Result<()> {
        for contact in contacts {
            self.update_user_info(&contact.id, |info| {
                info.name.roster = contact.name;
            })
            .await?;
        }
        Ok(())
    }

    async fn reset_before_reconnect(&self) -> Result<()> {
        self.requested_vcards.write().clear();
        self.requested_avatars.write().clear();
        self.user_profile_repo
            .reset_before_reconnect(&self.ctx.connected_account()?)
            .await
    }

    async fn clear_cache(&self) -> Result<()> {
        let account = self.ctx.connected_account()?;
        self.user_info_repo.clear_cache(&account).await?;
        self.user_profile_repo.clear_cache(&account).await?;
        Ok(())
    }
}

impl UserInfoDomainService {
    async fn update_user_info(
        &self,
        user_id: &UserId,
        handler: impl FnOnce(&mut UserInfo) + Send + 'static,
    ) -> Result<()> {
        let account = self.ctx.connected_account()?;
        let is_self_event = account == *user_id;

        let user_info_changed = self
            .user_info_repo
            .update(&account, user_id, Box::new(handler))
            .await?;

        // Let's not dispatch events if nothing has changed or we're not connected (yet)
        if !user_info_changed || self.ctx.connection_state() != ConnectionState::Connected {
            return Ok(());
        }

        self.client_event_dispatcher
            .dispatch_event(ClientEvent::ContactChanged {
                ids: vec![user_id.clone()],
            });

        if is_self_event {
            self.client_event_dispatcher
                .dispatch_event(ClientEvent::AccountInfoChanged)
        }

        Ok(())
    }

    async fn load_user_profile_and_update_user_info(
        &self,
        participant_id: ParticipantIdRef<'_>,
        cache_policy: CachePolicy,
    ) -> Result<Option<UserProfile>> {
        let account = self.ctx.connected_account()?;

        let cached_profile = self.user_profile_repo.get(&account, participant_id).await?;

        match cache_policy {
            _ if self
                .requested_vcards
                .read()
                .contains(participant_id.borrow()) =>
            {
                return Ok(cached_profile)
            }
            // Only the cached data was requested…
            CachePolicy::ReturnCacheDataDontLoad => return Ok(cached_profile),
            // We found cached data, so we'll return it…
            CachePolicy::ReturnCacheDataElseLoad if cached_profile.is_some() => {
                return Ok(cached_profile)
            }
            CachePolicy::ReturnCacheDataElseLoad => (),
        };

        // If the contact is blocked, we'll also only return the cached data…
        if let Some(user_id) = participant_id.to_user_id() {
            if self.block_list_repo.contains(&account, user_id).await? {
                return Ok(cached_profile);
            }
        }

        // Load either vCard4 or vCard temp depending on server capabilities and id we have at hand…
        let result = match participant_id {
            ParticipantIdRef::User(_) if self.ctx.server_features()?.vcard4 => {
                // We're not loading vCard4, because if we have the right to access it, we'll
                // receive it via push, or otherwise it wouldn't make sense to even try to load it.
                return Ok(cached_profile);
            }
            _ => self.user_info_service.load_vcard_temp(participant_id).await,
        };

        let user_profile = match result {
            Ok(user_profile) => user_profile,
            Err(err) => {
                self.requested_vcards
                    .write()
                    .insert(participant_id.as_ref().clone());
                return Err(err.into());
            }
        };

        self.requested_vcards
            .write()
            .insert(participant_id.as_ref().clone());

        if let Some(user_id) = participant_id.to_user_id() {
            self.handle_user_profile_changed(user_id, user_profile.clone())
                .await?;
        } else {
            self.user_profile_repo
                .set(&account, participant_id, user_profile.as_ref())
                .await?;
        }

        Ok(user_profile)
    }
}
