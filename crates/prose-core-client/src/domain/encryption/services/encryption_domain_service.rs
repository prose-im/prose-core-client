// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use async_trait::async_trait;

use prose_wasm_utils::{SendUnlessWasm, SyncUnlessWasm};

use crate::domain::encryption::models::{DecryptionContext, DeviceId, DeviceInfo, DeviceList};
use crate::domain::messaging::models::{EncryptedPayload, KeyTransportPayload, MessageId};
use crate::domain::shared::models::{RoomId, UserId};

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("The recipient does not have any OMEMO-enabled devices.")]
    NoDevices(UserId),
    #[error("The recipient does not have any trusted OMEMO-enabled devices.")]
    NoTrustedDevices(UserId),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum DecryptionError {
    #[error("The message was not encrypted for this device.")]
    NotEncryptedForThisDevice,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
#[cfg_attr(feature = "test", mockall::automock)]
pub trait EncryptionDomainService: SendUnlessWasm + SyncUnlessWasm {
    async fn initialize(&self) -> Result<()>;

    async fn encrypt_message(
        &self,
        recipient_ids: Vec<UserId>,
        message: String,
    ) -> Result<EncryptedPayload, EncryptionError>;

    /// Decrypts the payload and returns the decrypted message.
    /// - If the payload does not contain an encrypted message, processes the key material and
    ///   returns None.
    /// - If decrypting the message fails, tries to look up the decrypted message in the
    ///   MessagesRepository and returns it from there.
    async fn decrypt_message(
        &self,
        sender_id: &UserId,
        room_id: &RoomId,
        message_id: Option<&MessageId>,
        payload: EncryptedPayload,
        context: Option<DecryptionContext>,
    ) -> Result<String, DecryptionError>;

    async fn finalize_decryption(&self, context: DecryptionContext);

    async fn load_device_infos(&self, user_id: &UserId) -> Result<Vec<DeviceInfo>>;
    async fn delete_device(&self, device_id: &DeviceId) -> Result<()>;
    async fn disable_omemo(&self) -> Result<()>;

    async fn handle_received_key_transport_message(
        &self,
        sender_id: &UserId,
        payload: KeyTransportPayload,
        context: Option<DecryptionContext>,
    ) -> Result<()>;

    async fn handle_received_device_list(
        &self,
        user_id: &UserId,
        device_list: DeviceList,
    ) -> Result<()>;

    async fn reset_before_reconnect(&self) -> Result<()>;
    async fn clear_cache(&self) -> Result<()>;
}
