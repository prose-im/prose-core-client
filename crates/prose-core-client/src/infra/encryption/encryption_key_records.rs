// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use prose_store::prelude::Entity;
use prose_store::{define_entity, IndexSpec};

use crate::domain::encryption::models::{
    DeviceId, IdentityKey, IdentityKeyPair, KyberPreKey, KyberPreKeyId, LocalDevice, SenderKey,
    Session, SessionData, Trust,
};
use crate::domain::shared::models::AccountId;
use crate::dtos::{PreKey, PreKeyId, PrivateKey, PublicKey, SignedPreKey, SignedPreKeyId, UserId};

mod columns {
    pub const ACCOUNT: &str = "account";
    pub const USER_ID: &str = "user_id";
    pub const DEVICE_ID: &str = "device_id";
    pub const PRE_KEY_ID: &str = "pre_key_id";
    pub const DISTRIBUTION_ID: &str = "distribution_id";
}

#[derive(Serialize, Deserialize)]
pub struct LocalDeviceRecord {
    id: AccountId,
    device_id: DeviceId,
    identity_key_pair: IdentityKeyPair,
}

impl LocalDeviceRecord {
    pub fn new(
        account: &AccountId,
        device_id: &DeviceId,
        identity_key_pair: IdentityKeyPair,
    ) -> Self {
        Self {
            id: account.clone(),
            device_id: device_id.clone(),
            identity_key_pair,
        }
    }
}

impl From<LocalDeviceRecord> for LocalDevice {
    fn from(value: LocalDeviceRecord) -> Self {
        Self {
            device_id: value.device_id,
            identity_key_pair: value.identity_key_pair,
        }
    }
}

define_entity!(LocalDeviceRecord, "omemo_local_device", AccountId);

#[derive(Serialize, Deserialize)]
pub struct SignedPreKeyRecord {
    id: String,
    account: AccountId,
    pre_key_id: SignedPreKeyId,
    public_key: PublicKey,
    private_key: PrivateKey,
    signature: Box<[u8]>,
    timestamp: u64,
}

impl SignedPreKeyRecord {
    pub fn new(account: &AccountId, signed_pre_key: SignedPreKey) -> Self {
        Self {
            id: format!("{}.{}", account, signed_pre_key.id.as_ref()),
            account: account.clone(),
            pre_key_id: signed_pre_key.id,
            public_key: signed_pre_key.public_key,
            private_key: signed_pre_key.private_key,
            signature: signed_pre_key.signature,
            timestamp: signed_pre_key.timestamp,
        }
    }
}

impl From<SignedPreKeyRecord> for SignedPreKey {
    fn from(value: SignedPreKeyRecord) -> Self {
        Self {
            id: value.pre_key_id,
            public_key: value.public_key,
            private_key: value.private_key,
            signature: value.signature,
            timestamp: value.timestamp,
        }
    }
}

define_entity!(SignedPreKeyRecord, "omemo_signed_pre_key",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    pre_key_idx => { columns: [columns::ACCOUNT, columns::PRE_KEY_ID], unique: true }
);

#[derive(Serialize, Deserialize)]
pub struct PreKeyRecord {
    id: String,
    account: AccountId,
    pre_key_id: PreKeyId,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl PreKeyRecord {
    pub fn new(account: &AccountId, pre_key: PreKey) -> Self {
        Self {
            id: format!("{}.{}", account, pre_key.id.as_ref()),
            account: account.clone(),
            pre_key_id: pre_key.id,
            public_key: pre_key.public_key,
            private_key: pre_key.private_key,
        }
    }
}

impl From<PreKeyRecord> for PreKey {
    fn from(value: PreKeyRecord) -> Self {
        Self {
            id: value.pre_key_id,
            public_key: value.public_key,
            private_key: value.private_key,
        }
    }
}

define_entity!(PreKeyRecord, "omemo_pre_key",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    pre_key_idx => { columns: [columns::ACCOUNT, columns::PRE_KEY_ID], unique: true }
);

#[derive(Serialize, Deserialize)]
pub struct KyberPreKeyRecord {
    id: String,
    account: AccountId,
    pre_key_id: KyberPreKeyId,
    pre_key: KyberPreKey,
}

impl KyberPreKeyRecord {
    pub fn new(account: &AccountId, id: &KyberPreKeyId, pre_key: KyberPreKey) -> Self {
        Self {
            id: format!("{}.{}", account, id.as_ref()),
            account: account.clone(),
            pre_key_id: id.clone(),
            pre_key,
        }
    }
}

impl From<KyberPreKeyRecord> for KyberPreKey {
    fn from(value: KyberPreKeyRecord) -> Self {
        value.pre_key
    }
}

define_entity!(KyberPreKeyRecord, "omemo_kyber_pre_key",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    pre_key_idx => { columns: [columns::ACCOUNT, columns::PRE_KEY_ID], unique: true }
);

#[derive(Serialize, Deserialize)]
pub struct SenderKeyRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    device_id: DeviceId,
    distribution_id: Uuid,
    key: SenderKey,
}

impl SenderKeyRecord {
    pub fn new(
        account: &AccountId,
        user_id: &UserId,
        device_id: &DeviceId,
        distribution_id: Uuid,
        key: SenderKey,
    ) -> Self {
        Self {
            id: format!("{}.{}.{}.{}", account, user_id, device_id, distribution_id),
            account: account.clone(),
            user_id: user_id.clone(),
            device_id: device_id.clone(),
            distribution_id,
            key,
        }
    }
}

impl From<SenderKeyRecord> for SenderKey {
    fn from(value: SenderKeyRecord) -> Self {
        value.key
    }
}

define_entity!(SenderKeyRecord, "omemo_sender_key",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    distribution_idx => { columns: [columns::ACCOUNT, columns::USER_ID, columns::DEVICE_ID, columns::DISTRIBUTION_ID], unique: true }
);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRecord {
    id: String,
    account: AccountId,
    user_id: UserId,
    pub device_id: DeviceId,
    pub trust: Trust,
    pub is_active: bool,
    pub data: Option<SessionData>,
    pub identity: Option<IdentityKey>,
}

impl SessionRecord {
    pub fn new(account: &AccountId, session: Session) -> Self {
        Self {
            id: format!("{}.{}.{}", account, session.user_id, session.device_id),
            account: account.clone(),
            user_id: session.user_id,
            device_id: session.device_id,
            trust: session.trust,
            is_active: session.is_active,
            data: session.data,
            identity: session.identity,
        }
    }
}

define_entity!(SessionRecord, "omemo_session_record",
    account_idx => { columns: [columns::ACCOUNT], unique: false },
    user_idx => { columns: [columns::ACCOUNT, columns::USER_ID], unique: false },
    device_idx => { columns: [columns::ACCOUNT, columns::USER_ID, columns::DEVICE_ID], unique: true }
);

impl From<SessionRecord> for Session {
    fn from(value: SessionRecord) -> Self {
        Self {
            user_id: value.user_id,
            device_id: value.device_id,
            trust: value.trust,
            is_active: value.is_active,
            identity: value.identity,
            data: value.data,
        }
    }
}
