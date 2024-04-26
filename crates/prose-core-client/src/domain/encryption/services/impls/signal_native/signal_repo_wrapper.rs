// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use libsignal_protocol::error::Result;
use libsignal_protocol::{
    Direction, IdentityKey, IdentityKeyPair, IdentityKeyStore, KyberPreKeyId, KyberPreKeyRecord,
    KyberPreKeyStore, PreKeyId, PreKeyRecord, PreKeyStore, ProtocolAddress, SenderKeyRecord,
    SenderKeyStore, SessionRecord, SessionStore, SignedPreKeyId, SignedPreKeyRecord,
    SignedPreKeyStore,
};
use uuid::Uuid;

use crate::app::deps::{DynEncryptionKeysRepository, DynSessionRepository};

use super::signal_compat::{map_repo_error, ProtocolAddressExt, UnwindSafeError};

#[derive(Clone)]
pub struct SignalRepoWrapper {
    encryption_keys_repo: DynEncryptionKeysRepository,
    session_repo: DynSessionRepository,
}

impl SignalRepoWrapper {
    pub fn new(
        encryption_keys_repo: DynEncryptionKeysRepository,
        session_repo: DynSessionRepository,
    ) -> Self {
        Self {
            encryption_keys_repo,
            session_repo,
        }
    }
}

#[async_trait(? Send)]
impl SessionStore for SignalRepoWrapper {
    async fn load_session(&self, address: &ProtocolAddress) -> Result<Option<SessionRecord>> {
        Ok(self
            .session_repo
            .get_session(&address.prose_user_id()?, &address.prose_device_id())
            .await
            .map_err(map_repo_error)?
            .and_then(|session| session.data.as_ref().map(|data| data.try_into()))
            .transpose()?)
    }

    async fn store_session(
        &mut self,
        address: &ProtocolAddress,
        record: &SessionRecord,
    ) -> Result<()> {
        self.session_repo
            .put_session_data(
                &address.prose_user_id()?,
                &address.prose_device_id(),
                record.try_into()?,
            )
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }
}

#[async_trait(? Send)]
impl PreKeyStore for SignalRepoWrapper {
    async fn get_pre_key(&self, prekey_id: PreKeyId) -> Result<PreKeyRecord> {
        self.encryption_keys_repo
            .get_pre_key(prekey_id.into())
            .await
            .map_err(map_repo_error)?
            .ok_or(libsignal_protocol::error::SignalProtocolError::InvalidPreKeyId.into())
            .and_then(|record| (&record).try_into())
    }

    async fn save_pre_key(&mut self, _prekey_id: PreKeyId, record: &PreKeyRecord) -> Result<()> {
        self.encryption_keys_repo
            .put_pre_keys(&[record.try_into()?])
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }

    async fn remove_pre_key(&mut self, prekey_id: PreKeyId) -> Result<()> {
        self.encryption_keys_repo
            .delete_pre_key(prekey_id.into())
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }
}

#[async_trait(? Send)]
impl SignedPreKeyStore for SignalRepoWrapper {
    async fn get_signed_pre_key(
        &self,
        signed_prekey_id: SignedPreKeyId,
    ) -> Result<SignedPreKeyRecord> {
        self.encryption_keys_repo
            .get_signed_pre_key(signed_prekey_id.into())
            .await
            .map_err(map_repo_error)?
            .ok_or(libsignal_protocol::error::SignalProtocolError::InvalidSignedPreKeyId)
            .and_then(|record| (&record).try_into())
    }

    async fn save_signed_pre_key(
        &mut self,
        _signed_prekey_id: SignedPreKeyId,
        record: &SignedPreKeyRecord,
    ) -> Result<()> {
        self.encryption_keys_repo
            .put_signed_pre_key(&record.try_into()?)
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }
}

#[async_trait(? Send)]
impl SenderKeyStore for SignalRepoWrapper {
    async fn store_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
        record: &SenderKeyRecord,
    ) -> Result<()> {
        self.encryption_keys_repo
            .put_sender_key(
                &sender.prose_user_id()?,
                &sender.prose_device_id(),
                distribution_id,
                &record.try_into()?,
            )
            .await
            .map_err(map_repo_error)
    }

    async fn load_sender_key(
        &mut self,
        sender: &ProtocolAddress,
        distribution_id: Uuid,
    ) -> Result<Option<SenderKeyRecord>> {
        Ok(self
            .encryption_keys_repo
            .get_sender_key(
                &sender.prose_user_id()?,
                &sender.prose_device_id(),
                distribution_id,
            )
            .await
            .map_err(map_repo_error)?
            .map(|record| (&record).try_into())
            .transpose()?)
    }
}

#[async_trait(? Send)]
impl IdentityKeyStore for SignalRepoWrapper {
    async fn get_identity_key_pair(&self) -> Result<IdentityKeyPair> {
        let Some(local_device) = self
            .encryption_keys_repo
            .get_local_device()
            .await
            .map_err(map_repo_error)?
        else {
            return Err(
                libsignal_protocol::error::SignalProtocolError::ApplicationCallbackError(
                    "Application Error",
                    Box::new(UnwindSafeError("Missing identity key pair".to_string())),
                ),
            );
        };
        Ok((&local_device.identity_key_pair).try_into()?)
    }

    async fn get_local_registration_id(&self) -> Result<u32> {
        let Some(local_device) = self
            .encryption_keys_repo
            .get_local_device()
            .await
            .map_err(map_repo_error)?
        else {
            return Err(
                libsignal_protocol::error::SignalProtocolError::ApplicationCallbackError(
                    "Application Error",
                    Box::new(UnwindSafeError("Missing device ID".to_string())),
                ),
            );
        };
        Ok(*local_device.device_id.as_ref())
    }

    async fn save_identity(
        &mut self,
        address: &ProtocolAddress,
        identity: &IdentityKey,
    ) -> Result<bool> {
        let did_exist = self
            .session_repo
            .put_identity(
                &address.prose_user_id()?,
                &address.prose_device_id(),
                identity.try_into()?,
            )
            .await
            .map_err(map_repo_error)?;
        Ok(did_exist)
    }

    async fn is_trusted_identity(
        &self,
        _address: &ProtocolAddress,
        _identity: &IdentityKey,
        _direction: Direction,
    ) -> Result<bool> {
        // We handle trust outside of libsignal. Meaning that we always want to decrypt received
        // messages and do not encrypt messages at all for untrusted devices.
        Ok(true)
    }

    async fn get_identity(&self, address: &ProtocolAddress) -> Result<Option<IdentityKey>> {
        let identity = self
            .session_repo
            .get_session(&address.prose_user_id()?, &address.prose_device_id())
            .await
            .map_err(map_repo_error)?
            .and_then(|session| {
                session
                    .identity
                    .as_ref()
                    .map(|identity| identity.try_into())
            })
            .transpose()?;
        Ok(identity)
    }
}

#[async_trait(? Send)]
impl KyberPreKeyStore for SignalRepoWrapper {
    async fn get_kyber_pre_key(&self, kyber_prekey_id: KyberPreKeyId) -> Result<KyberPreKeyRecord> {
        self.encryption_keys_repo
            .get_kyber_pre_key(kyber_prekey_id.into())
            .await
            .map_err(map_repo_error)?
            .ok_or(libsignal_protocol::error::SignalProtocolError::InvalidKyberPreKeyId)
            .and_then(|record| (&record).try_into())
    }

    async fn save_kyber_pre_key(
        &mut self,
        kyber_prekey_id: KyberPreKeyId,
        record: &KyberPreKeyRecord,
    ) -> Result<()> {
        self.encryption_keys_repo
            .put_kyber_pre_key(kyber_prekey_id.into(), &record.try_into()?)
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }

    async fn mark_kyber_pre_key_used(&mut self, kyber_prekey_id: KyberPreKeyId) -> Result<()> {
        self.encryption_keys_repo
            .delete_kyber_pre_key(kyber_prekey_id.into())
            .await
            .map_err(map_repo_error)?;
        Ok(())
    }
}
