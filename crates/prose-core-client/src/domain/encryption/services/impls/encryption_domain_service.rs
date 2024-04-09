// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;
use std::time::SystemTime;

use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes128Gcm, KeyInit};
use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use futures::future::join_all;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use tracing::{error, info, warn};

use prose_proc_macros::DependenciesStruct;
use prose_xmpp::TimeProvider;

use crate::app::deps::{
    DynAppContext, DynEncryptionKeysRepository, DynEncryptionService, DynMessagesRepository,
    DynTimeProvider, DynUserDeviceIdProvider, DynUserDeviceRepository, DynUserDeviceService,
};
use crate::domain::encryption::models::{Device, DeviceId, DeviceInfo, DeviceList, PreKeyBundle};
use crate::domain::messaging::models::send_message_request::EncryptedPayload;
use crate::domain::messaging::models::MessageLikePayload;
use crate::domain::shared::models::UserId;
use crate::dtos::{DeviceBundle, EncryptionDirection, MessageId, PreKeyId, RoomId};

use super::super::EncryptionDomainService as EncryptionDomainServiceTrait;

#[derive(DependenciesStruct)]
pub struct EncryptionDomainService {
    ctx: DynAppContext,
    encryption_keys_repo: DynEncryptionKeysRepository,
    encryption_service: DynEncryptionService,
    message_repo: DynMessagesRepository,
    time_provider: DynTimeProvider,
    user_device_id_provider: DynUserDeviceIdProvider,
    user_device_repo: DynUserDeviceRepository,
    user_device_service: DynUserDeviceService,
}

const KEY_SIZE: usize = 16;
const MAC_SIZE: usize = 16;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl EncryptionDomainServiceTrait for EncryptionDomainService {
    async fn initialize(&self) -> Result<()> {
        let bundle = self.get_or_initialize_local_encryption_keys().await?;
        self.publish_device_info_if_needed(bundle).await?;

        let user_id = self.ctx.connected_id()?.into_user_id();
        let devices = self.user_device_repo.get_all(&user_id).await?;

        join_all(devices.into_iter().map(|device| {
            let user_id = user_id.clone();
            async move {
                match self
                    .start_session_with_device(user_id.clone(), device.id.clone())
                    .await
                {
                    Ok(_) => (),
                    Err(err) => warn!(
                        "Failed to start OMEMO session with {user_id}'s device {}. Reason: {}",
                        device, err
                    ),
                }
            }
        }))
        .await;

        Ok(())
    }

    async fn start_session(&self, user_id: &UserId) -> Result<()> {
        let devices = self.user_device_repo.get_all(user_id).await?;

        join_all(
            devices
                .into_iter()
                .map(|device| self.start_session_with_device(user_id.clone(), device.id)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

        Ok(())
    }

    async fn encrypt_message(
        &self,
        recipient_id: &UserId,
        message: String,
    ) -> Result<EncryptedPayload> {
        let current_user_id = self.ctx.connected_id()?.into_user_id();

        let local_device = self
            .encryption_keys_repo
            .get_local_device()
            .await?
            .ok_or(anyhow!("Missing local encryption bundle"))?;

        let nonce = Aes128Gcm::generate_nonce(&mut OsRng);
        let dek = Aes128Gcm::generate_key(OsRng);
        let cipher = Aes128Gcm::new(&dek);

        let payload = cipher
            .encrypt(&nonce, message.as_bytes())
            .map_err(|err| anyhow!("{err}"))?;

        let mut dek_and_mac = [0u8; KEY_SIZE + MAC_SIZE];
        dek_and_mac[..KEY_SIZE].copy_from_slice(&dek);
        dek_and_mac[KEY_SIZE..KEY_SIZE + MAC_SIZE].copy_from_slice(&payload[message.len()..]);

        let now = SystemTime::from(self.time_provider.now());

        // Instead of encrypting the message for all the user's devices we'll only encrypt it
        // for devices which we have an active session with, i.e. devices that are actually trusted.
        // Otherwise, libsignal will choke later on.
        let encrypt_message_futures = self
            .encryption_keys_repo
            .get_active_device_ids(&current_user_id)
            .await?
            .into_iter()
            .filter(|device_id| device_id != &local_device.device_id)
            .map(|device_id| (&current_user_id, device_id))
            .chain(
                self.encryption_keys_repo
                    .get_active_device_ids(&recipient_id)
                    .await?
                    .into_iter()
                    .map(|device_id| (recipient_id, device_id)),
            )
            .map(|(user_id, device_id)| async move {
                self.encryption_service
                    .encrypt_key(user_id, &device_id, &dek_and_mac, &now)
                    .await
            });

        let messages = join_all(encrypt_message_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let payload = EncryptedPayload {
            device_id: local_device.device_id,
            iv: nonce.as_slice().into(),
            messages,
            payload: payload[..message.len()].into(),
        };

        Ok(payload)
    }

    async fn decrypt_message(
        &self,
        sender_id: &UserId,
        message_id: &MessageId,
        payload: EncryptedPayload,
    ) -> Result<String> {
        let error = match self._decrypt_message(sender_id, payload).await {
            Ok(message) => return Ok(message),
            Err(error) => error,
        };

        let Ok(messages) = self
            .message_repo
            .get(&RoomId::User(sender_id.clone()), message_id)
            .await
        else {
            return Err(error);
        };

        let Some(message) = messages.first() else {
            return Err(error);
        };

        let MessageLikePayload::Message { body, .. } = &message.payload else {
            return Err(error);
        };

        Ok(body.to_string())
    }

    async fn load_device_infos(&self, user_id: &UserId) -> Result<Vec<DeviceInfo>> {
        let this_device_id = if &self.ctx.connected_id()?.into_user_id() == user_id {
            self.encryption_keys_repo
                .get_local_device()
                .await?
                .map(|device| device.device_id)
        } else {
            None
        };

        let device_list = self.user_device_repo.get_all(user_id).await?;

        let mut device_infos = vec![];
        for device in device_list {
            let Some(identity) = self
                .encryption_keys_repo
                .get_identity(user_id, &device.id)
                .await?
            else {
                warn!(
                    "Ignoring device {} for which we do not have an identity.",
                    device.id
                );
                continue;
            };

            let is_device_trusted = self
                .encryption_keys_repo
                .is_trusted_identity(
                    user_id,
                    Some(&device.id),
                    &identity,
                    EncryptionDirection::Receiving,
                )
                .await?;

            let is_this_device = Some(&device.id) == this_device_id.as_ref();

            device_infos.push(DeviceInfo {
                id: device.id,
                label: device.label,
                identity,
                is_trusted: is_device_trusted,
                is_this_device,
            });
        }

        Ok(device_infos)
    }

    async fn delete_device(&self, device_id: &DeviceId) -> Result<()> {
        let user_id = self.ctx.connected_id()?.into_user_id();

        let mut devices = self.user_device_repo.get_all(&user_id).await?;
        let num_devices = devices.len();
        devices.retain(|device| &device.id != device_id);

        if devices.len() == num_devices {
            return Ok(());
        }

        self.user_device_repo
            .set_all(&user_id, devices.clone())
            .await?;

        self.user_device_service
            .publish_device_list(DeviceList { devices })
            .await?;

        self.user_device_service
            .delete_device_bundle(device_id)
            .await?;

        Ok(())
    }

    async fn disable_omemo(&self) -> Result<()> {
        let devices = self
            .user_device_repo
            .get_all(&self.ctx.connected_id()?.into_user_id())
            .await?;

        self.user_device_service.delete_device_list().await?;

        for device in devices {
            _ = self
                .user_device_service
                .delete_device_bundle(&device.id)
                .await
        }

        Ok(())
    }

    async fn handle_received_device_list(
        &self,
        user_id: &UserId,
        device_list: DeviceList,
    ) -> Result<()> {
        self.user_device_repo
            .set_all(user_id, device_list.devices)
            .await?;
        Ok(())
    }

    async fn clear_cache(&self) -> Result<()> {
        self.user_device_repo.clear_cache().await?;
        self.encryption_keys_repo.clear_cache().await?;
        Ok(())
    }
}

impl EncryptionDomainService {
    async fn get_or_initialize_local_encryption_keys(&self) -> Result<DeviceBundle> {
        if let Some(local_encryption_bundle) =
            self.encryption_keys_repo.get_local_device_bundle().await?
        {
            return Ok(local_encryption_bundle);
        }

        let local_encryption_bundle = self
            .encryption_service
            .generate_local_encryption_bundle(self.user_device_id_provider.new_id())
            .await?;

        self.encryption_keys_repo
            .put_local_encryption_bundle(&local_encryption_bundle)
            .await?;

        Ok(local_encryption_bundle.into_device_bundle())
    }

    async fn start_session_with_device(&self, user_id: UserId, device_id: DeviceId) -> Result<()> {
        info!("Starting OMEMO session with {user_id} ({device_id})…");

        let Some(bundle) = self
            .user_device_service
            .load_device_bundle(&user_id, &device_id)
            .await?
        else {
            info!("No device bundle found for {user_id} ({device_id}).");
            return Ok(());
        };

        self.encryption_keys_repo
            .save_identity(&user_id, &device_id, &bundle.identity_key)
            .await?;

        let pre_key_bundle = PreKeyBundle {
            device_id,
            signed_pre_key: bundle.signed_pre_key,
            identity_key: bundle.identity_key,
            pre_key: bundle
                .pre_keys
                .choose(&mut thread_rng())
                .ok_or(anyhow!("No pre_keys available."))?
                .clone(),
        };

        self.encryption_service
            .process_pre_key_bundle(&user_id, pre_key_bundle)
            .await?;

        Ok(())
    }

    async fn generate_missing_pre_keys(&self) -> Result<()> {
        let pre_keys = self.encryption_keys_repo.get_all_pre_keys().await?;

        let pre_key_ids = pre_keys
            .iter()
            .map(|pre_key| pre_key.id.as_ref())
            .collect::<HashSet<_>>();
        let missing_pre_key_ids = (1..=100)
            .filter_map(|idx| {
                if pre_key_ids.contains(&idx) {
                    return None;
                }
                return Some(PreKeyId::from(idx));
            })
            .collect::<Vec<_>>();

        if missing_pre_key_ids.is_empty() {
            return Ok(());
        }

        info!("Generating {} new PreKeys…", missing_pre_key_ids.len());
        let missing_pre_keys = self
            .encryption_service
            .generate_pre_keys_with_ids(missing_pre_key_ids)
            .await?;

        info!("Saving new PreKeys…");
        self.encryption_keys_repo
            .put_pre_keys(missing_pre_keys.as_slice())
            .await?;

        info!("Publishing bundle with new PreKeys…");
        let bundle = self
            .encryption_keys_repo
            .get_local_device_bundle()
            .await?
            .ok_or(anyhow!("Missing own device bundle"))?;
        self.user_device_service
            .publish_device_bundle(bundle)
            .await?;

        Ok(())
    }

    async fn publish_device_info_if_needed(&self, bundle: DeviceBundle) -> Result<()> {
        let user_id = self.ctx.connected_id()?.into_user_id();

        let mut devices = self.user_device_repo.get_all(&user_id).await?;
        // Add our device to our device list if needed…
        if !devices
            .iter()
            .find(|device| device.id == bundle.device_id)
            .is_some()
        {
            info!(
                "Adding our device {} the list of devices…",
                bundle.device_id
            );
            devices.push(Device {
                id: bundle.device_id.clone(),
                label: Some(
                    self.ctx
                        .software_version
                        .os
                        .as_ref()
                        .map(|os| format!("{} ({})", self.ctx.software_version.name, os))
                        .unwrap_or(self.ctx.software_version.name.clone()),
                ),
            });
            self.user_device_service
                .publish_device_list(DeviceList { devices })
                .await?;
        }

        let published_bundle = self
            .user_device_service
            .load_device_bundle(&user_id, &bundle.device_id)
            .await?;

        // … and publish our device bundle…
        if published_bundle.is_none() {
            info!("Publishing our device bundle…");
            self.user_device_service
                .publish_device_bundle(bundle)
                .await?;
        }

        Ok(())
    }

    async fn _decrypt_message(
        &self,
        sender_id: &UserId,
        payload: EncryptedPayload,
    ) -> Result<String> {
        let local_device = self
            .encryption_keys_repo
            .get_local_device()
            .await?
            .ok_or(anyhow!("Missing local encryption bundle"))?;

        let encrypted_message = payload
            .messages
            .into_iter()
            .find(|message| message.device_id == local_device.device_id)
            .ok_or(anyhow!("Message was not encrypted for current device."))?;

        let dek_and_mac = self
            .encryption_service
            .decrypt_key(
                sender_id,
                &payload.device_id,
                &encrypted_message.data.as_ref(),
                encrypted_message.prekey,
            )
            .await?;

        if dek_and_mac.len() != MAC_SIZE + KEY_SIZE {
            bail!("Invalid DEK and MAC size");
        }

        let dek = aes_gcm::Key::<Aes128Gcm>::from_slice(&dek_and_mac[..KEY_SIZE]);
        let mac = &dek_and_mac[KEY_SIZE..KEY_SIZE + MAC_SIZE];
        let mut payload_and_mac = Vec::with_capacity(payload.payload.len() + mac.len());
        payload_and_mac.extend_from_slice(payload.payload.as_ref());
        payload_and_mac.extend(mac);

        let cipher = Aes128Gcm::new(&dek);
        let nonce =
            aes_gcm::Nonce::<<Aes128Gcm as AeadCore>::NonceSize>::from_slice(payload.iv.as_ref());
        let message = String::from_utf8(
            cipher
                .decrypt(nonce, payload_and_mac.as_slice())
                .map_err(|err| anyhow!("{err}"))?,
        )?;

        if encrypted_message.prekey {
            if let Err(err) = self.generate_missing_pre_keys().await {
                error!("Failed to generate missing prekeys. {}", err.to_string())
            }
        }

        Ok(message)
    }
}
