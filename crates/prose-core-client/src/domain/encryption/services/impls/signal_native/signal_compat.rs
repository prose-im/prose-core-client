// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use libsignal_protocol::error::{Result as SignalResult, SignalProtocolError as SignalError};
use libsignal_protocol::GenericSignedPreKey;
use tracing::error;

use crate::domain::encryption::models::{
    DeviceId, IdentityKey, IdentityKeyPair, KyberPreKey, KyberPreKeyId, PreKey, PreKeyId,
    PrivateKey, PublicKey, SenderKey, SessionData, SignedPreKey, SignedPreKeyId,
};
use crate::dtos::UserId;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct UnwindSafeError(pub String);

pub fn map_repo_error(error: anyhow::Error) -> SignalError {
    error.downcast::<SignalError>().unwrap_or_else(|error| {
        SignalError::ApplicationCallbackError(
            "EncryptionKeysRepoError",
            Box::new(UnwindSafeError(error.to_string())),
        )
    })
}

pub trait ProtocolAddressExt {
    fn prose_user_id(&self) -> SignalResult<UserId>;
    fn prose_device_id(&self) -> DeviceId;
}

impl ProtocolAddressExt for libsignal_protocol::ProtocolAddress {
    fn prose_user_id(&self) -> SignalResult<UserId> {
        self.name().parse().map_err(|err: jid::Error| {
            libsignal_protocol::error::SignalProtocolError::ApplicationCallbackError(
                "UserId Parse Error",
                Box::new(UnwindSafeError(err.to_string())),
            )
        })
    }

    fn prose_device_id(&self) -> DeviceId {
        self.device_id().into()
    }
}

impl TryFrom<&PublicKey> for libsignal_protocol::PublicKey {
    type Error = SignalError;

    fn try_from(value: &PublicKey) -> SignalResult<Self> {
        Self::deserialize(value.as_ref())
    }
}

impl TryFrom<&libsignal_protocol::PublicKey> for PublicKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::PublicKey) -> SignalResult<Self> {
        let data = value.serialize();
        PublicKey::try_from(data.as_ref()).map_err(|err| {
            SignalError::ApplicationCallbackError(
                "Conversion Error",
                Box::new(UnwindSafeError(err.to_string())),
            )
        })
    }
}

impl TryFrom<&PrivateKey> for libsignal_protocol::PrivateKey {
    type Error = SignalError;

    fn try_from(value: &PrivateKey) -> SignalResult<Self> {
        Self::deserialize(value.as_ref())
    }
}

impl TryFrom<&libsignal_protocol::PrivateKey> for PrivateKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::PrivateKey) -> SignalResult<Self> {
        let data = value.serialize();
        PrivateKey::try_from(data.as_slice()).map_err(|err| {
            SignalError::ApplicationCallbackError(
                "Conversion Error",
                Box::new(UnwindSafeError(err.to_string())),
            )
        })
    }
}

impl TryFrom<&IdentityKey> for libsignal_protocol::IdentityKey {
    type Error = SignalError;

    fn try_from(value: &IdentityKey) -> SignalResult<Self> {
        Ok(Self::new(value.as_ref().try_into()?))
    }
}

impl TryFrom<&libsignal_protocol::IdentityKey> for IdentityKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::IdentityKey) -> SignalResult<Self> {
        Ok(Self::from(PublicKey::try_from(value.public_key())?))
    }
}

impl From<PreKeyId> for libsignal_protocol::PreKeyId {
    fn from(value: PreKeyId) -> Self {
        Self::from(value.into_inner())
    }
}

impl From<libsignal_protocol::PreKeyId> for PreKeyId {
    fn from(value: libsignal_protocol::PreKeyId) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<SignedPreKeyId> for libsignal_protocol::SignedPreKeyId {
    fn from(value: SignedPreKeyId) -> Self {
        Self::from(value.into_inner())
    }
}

impl From<libsignal_protocol::SignedPreKeyId> for SignedPreKeyId {
    fn from(value: libsignal_protocol::SignedPreKeyId) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<KyberPreKeyId> for libsignal_protocol::KyberPreKeyId {
    fn from(value: KyberPreKeyId) -> Self {
        Self::from(value.into_inner())
    }
}

impl From<libsignal_protocol::KyberPreKeyId> for KyberPreKeyId {
    fn from(value: libsignal_protocol::KyberPreKeyId) -> Self {
        Self::from(u32::from(value))
    }
}

impl TryFrom<&libsignal_protocol::IdentityKeyPair> for IdentityKeyPair {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::IdentityKeyPair) -> SignalResult<Self> {
        Ok(Self {
            identity_key: value.identity_key().try_into()?,
            private_key: value.private_key().try_into()?,
        })
    }
}

impl TryFrom<&IdentityKeyPair> for libsignal_protocol::IdentityKeyPair {
    type Error = SignalError;

    fn try_from(value: &IdentityKeyPair) -> SignalResult<Self> {
        Ok(Self::new(
            (&value.identity_key).try_into()?,
            (&value.private_key).try_into()?,
        ))
    }
}

impl From<libsignal_protocol::DeviceId> for DeviceId {
    fn from(value: libsignal_protocol::DeviceId) -> Self {
        DeviceId::from(u32::from(value))
    }
}

impl From<DeviceId> for libsignal_protocol::DeviceId {
    fn from(value: DeviceId) -> Self {
        libsignal_protocol::DeviceId::from(value.into_inner())
    }
}

impl TryFrom<&libsignal_protocol::PreKeyRecord> for PreKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::PreKeyRecord) -> SignalResult<Self> {
        Ok(Self {
            id: value.id()?.into(),
            public_key: (&value.public_key()?).try_into()?,
            private_key: (&value.private_key()?).try_into()?,
        })
    }
}

impl TryFrom<&PreKey> for libsignal_protocol::PreKeyRecord {
    type Error = SignalError;

    fn try_from(value: &PreKey) -> SignalResult<Self> {
        Ok(Self::new(
            value.id.into(),
            &libsignal_protocol::KeyPair::new(
                (&value.public_key).try_into()?,
                (&value.private_key).try_into()?,
            ),
        ))
    }
}

impl TryFrom<&libsignal_protocol::SignedPreKeyRecord> for SignedPreKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::SignedPreKeyRecord) -> SignalResult<Self> {
        Ok(Self {
            id: value.id()?.into(),
            public_key: (&value.public_key()?).try_into()?,
            private_key: (&value.private_key()?).try_into()?,
            signature: value.signature()?.into(),
            timestamp: value.timestamp()?,
        })
    }
}

impl TryFrom<&SignedPreKey> for libsignal_protocol::SignedPreKeyRecord {
    type Error = SignalError;

    fn try_from(value: &SignedPreKey) -> SignalResult<Self> {
        Ok(Self::new(
            value.id.into(),
            value.timestamp,
            &libsignal_protocol::KeyPair::new(
                (&value.public_key).try_into()?,
                (&value.private_key).try_into()?,
            ),
            value.signature.as_ref(),
        ))
    }
}

impl TryFrom<&libsignal_protocol::KyberPreKeyRecord> for KyberPreKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::KyberPreKeyRecord) -> SignalResult<Self> {
        Ok(Self::from(value.serialize()?.into_boxed_slice()))
    }
}

impl TryFrom<&KyberPreKey> for libsignal_protocol::KyberPreKeyRecord {
    type Error = SignalError;

    fn try_from(value: &KyberPreKey) -> SignalResult<Self> {
        Self::deserialize(value.as_ref())
    }
}

impl TryFrom<&libsignal_protocol::SenderKeyRecord> for SenderKey {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::SenderKeyRecord) -> SignalResult<Self> {
        Ok(Self::from(value.serialize()?.into_boxed_slice()))
    }
}

impl TryFrom<&SenderKey> for libsignal_protocol::SenderKeyRecord {
    type Error = SignalError;

    fn try_from(value: &SenderKey) -> SignalResult<Self> {
        Self::deserialize(value.as_ref())
    }
}

impl TryFrom<&libsignal_protocol::SessionRecord> for SessionData {
    type Error = SignalError;

    fn try_from(value: &libsignal_protocol::SessionRecord) -> SignalResult<Self> {
        Ok(Self::from(value.serialize()?.into_boxed_slice()))
    }
}

impl TryFrom<&SessionData> for libsignal_protocol::SessionRecord {
    type Error = SignalError;

    fn try_from(value: &SessionData) -> SignalResult<Self> {
        Self::deserialize(value.as_ref())
    }
}
