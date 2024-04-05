// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_core_client::dtos::{
    EncryptionDirection, IdentityKey, IdentityKeyPair, PreKeyBundle, PreKeyRecord, PrivateKey,
    PublicKey, SignedPreKeyRecord,
};

use crate::encryption::signal_repo::{
    Direction, PreKeyBundle as JsPreKeyBundle, PreKeyPairType, PreKeyType, SignedPreKeyPairType,
    SignedPublicPreKeyType,
};

use super::KeyPairType;

impl From<KeyPairType> for IdentityKeyPair {
    fn from(value: KeyPairType) -> Self {
        Self {
            identity_key: IdentityKey::from(value.public_key.as_ref()),
            private_key: PrivateKey::from(value.private_key.as_ref()),
        }
    }
}

impl From<IdentityKeyPair> for KeyPairType {
    fn from(value: IdentityKeyPair) -> Self {
        Self {
            public_key: value.identity_key.into_inner(),
            private_key: value.private_key.into_inner(),
        }
    }
}

impl From<SignedPreKeyPairType> for SignedPreKeyRecord {
    fn from(value: SignedPreKeyPairType) -> Self {
        Self {
            id: value.key_id().into(),
            public_key: PublicKey::from(value.key_pair.public_key.as_ref()),
            private_key: PrivateKey::from(value.key_pair.private_key.as_ref()),
            signature: value.signature,
            timestamp: 0,
        }
    }
}

impl From<PreKeyPairType> for PreKeyRecord {
    fn from(value: PreKeyPairType) -> Self {
        Self {
            id: value.key_id.into(),
            public_key: PublicKey::from(value.key_pair.public_key.as_ref()),
            private_key: PrivateKey::from(value.key_pair.private_key.as_ref()),
        }
    }
}

impl From<PreKeyRecord> for KeyPairType {
    fn from(value: PreKeyRecord) -> Self {
        Self {
            public_key: value.public_key.into_inner(),
            private_key: value.private_key.into_inner(),
        }
    }
}

impl From<SignedPreKeyRecord> for KeyPairType {
    fn from(value: SignedPreKeyRecord) -> Self {
        Self {
            public_key: value.public_key.into_inner(),
            private_key: value.private_key.into_inner(),
        }
    }
}

impl From<PreKeyBundle> for JsPreKeyBundle {
    fn from(value: PreKeyBundle) -> Self {
        Self {
            identity_key: value.identity_key.as_ref().into(),
            signed_pre_key: SignedPublicPreKeyType {
                key_id: value.signed_pre_key.id.into_inner(),
                public_key: value.signed_pre_key.key.as_ref().into(),
                signature: value.signed_pre_key.signature.into(),
            },
            pre_key: PreKeyType {
                key_id: value.pre_key.id.into_inner(),
                public_key: value.pre_key.key.as_ref().into(),
            },
            registration_id: value.device_id.into_inner(),
        }
    }
}

impl From<Direction> for EncryptionDirection {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Sending => EncryptionDirection::Sending,
            Direction::Receiving => EncryptionDirection::Receiving,
        }
    }
}
