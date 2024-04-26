// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::anyhow;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use wasm_bindgen_derive::TryFromJsValue;

use prose_core_client::dtos::{
    IdentityKey, PreKeyId, PreKeyRecord, PrivateKey, PublicKey, SessionData, SignedPreKeyId,
    SignedPreKeyRecord,
};
use prose_core_client::{DynEncryptionKeysRepository, DynSessionRepository};

use crate::encryption::try_decode_address;
use crate::error::{Result, WasmError};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct KeyPairType {
    #[wasm_bindgen(skip)]
    pub public_key: Box<[u8]>,
    #[wasm_bindgen(skip)]
    pub private_key: Box<[u8]>,
}

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PreKeyPairType {
    #[wasm_bindgen(skip)]
    pub key_id: u32,
    #[wasm_bindgen(skip)]
    pub key_pair: KeyPairType,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SignedPreKeyPairType {
    #[wasm_bindgen(skip)]
    pub key_id: u32,
    #[wasm_bindgen(skip)]
    pub key_pair: KeyPairType,
    #[wasm_bindgen(skip)]
    pub signature: Box<[u8]>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PreKeyType {
    #[wasm_bindgen(skip)]
    pub key_id: u32,
    #[wasm_bindgen(skip)]
    pub public_key: Box<[u8]>,
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct SignedPublicPreKeyType {
    #[wasm_bindgen(skip)]
    pub key_id: u32,
    #[wasm_bindgen(skip)]
    pub public_key: Box<[u8]>,
    #[wasm_bindgen(skip)]
    pub signature: Box<[u8]>,
}

#[wasm_bindgen]
pub enum Direction {
    Sending = 1,
    Receiving = 2,
}

#[wasm_bindgen]
pub struct PreKeyBundle {
    #[wasm_bindgen(skip)]
    pub identity_key: Box<[u8]>,
    #[wasm_bindgen(skip)]
    pub signed_pre_key: SignedPublicPreKeyType,
    #[wasm_bindgen(skip)]
    pub pre_key: PreKeyType,
    #[wasm_bindgen(skip)]
    pub registration_id: u32,
}

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct LocalEncryptionBundle {
    #[wasm_bindgen(skip)]
    pub identity_key: KeyPairType,
    #[wasm_bindgen(skip)]
    pub signed_pre_key: SignedPreKeyPairType,
    #[wasm_bindgen(skip)]
    pub pre_keys: Vec<PreKeyPairType>,
}

#[derive(TryFromJsValue)]
#[wasm_bindgen]
#[derive(Clone)]
pub struct EncryptedMessage {
    #[wasm_bindgen(skip)]
    pub data: Box<[u8]>,
    #[wasm_bindgen(skip)]
    pub prekey: bool,
}

pub type SessionRecordType = String;

#[wasm_bindgen]
#[derive(Clone)]
pub struct SignalRepo {
    encryption_keys_repo: DynEncryptionKeysRepository,
    session_repo: DynSessionRepository,
}

impl SignalRepo {
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

#[wasm_bindgen]
impl SignalRepo {
    #[wasm_bindgen(js_name = "getIdentityKeyPair")]
    pub async fn get_identity_key_pair(&self) -> Result<Option<KeyPairType>> {
        let key_pair = self
            .encryption_keys_repo
            .get_local_device()
            .await
            .map_err(WasmError::from)?
            .map(|device| KeyPairType::from(device.identity_key_pair));
        Ok(key_pair)
    }

    #[wasm_bindgen(js_name = "getLocalRegistrationId")]
    pub async fn get_local_registration_id(&self) -> Result<Option<u32>> {
        let registration_id = self
            .encryption_keys_repo
            .get_local_device()
            .await
            .map_err(WasmError::from)?
            .map(|device| device.device_id.into_inner());
        Ok(registration_id)
    }

    #[wasm_bindgen(js_name = "isTrustedIdentity")]
    pub async fn is_trusted_identity(
        &self,
        _identifier: &str,
        _identity_key: &[u8],
        _direction: Direction,
    ) -> Result<bool> {
        // We handle trust outside of libsignal. Meaning that we always want to decrypt received
        // messages and do not encrypt messages at all for untrusted devices.
        Ok(true)
    }

    #[wasm_bindgen(js_name = "saveIdentity")]
    pub async fn save_identity(
        &self,
        encoded_address: &str,
        public_key: &[u8],
        _non_blocking_approval: Option<bool>,
    ) -> Result<bool> {
        let (user_id, device_id) = try_decode_address(encoded_address).map_err(WasmError::from)?;
        let did_exist = self
            .session_repo
            .put_identity(&user_id, &device_id, IdentityKey::from(public_key))
            .await
            .map_err(WasmError::from)?;
        Ok(did_exist)
    }

    #[wasm_bindgen(js_name = "loadPreKey")]
    pub async fn load_pre_key(&self, encoded_address: &JsValue) -> Result<Option<KeyPairType>> {
        let pre_key_id = encoded_address
            .as_f64()
            .ok_or(anyhow!("Invalid pre_key id {:?}", encoded_address))
            .map(|value| PreKeyId::from(value as u32))
            .map_err(WasmError::from)?;

        let key_pair = self
            .encryption_keys_repo
            .get_pre_key(pre_key_id)
            .await
            .map_err(WasmError::from)?
            .map(KeyPairType::from);

        Ok(key_pair)
    }

    #[wasm_bindgen(js_name = "storePreKey")]
    pub async fn store_pre_key(&self, key_id: &JsValue, key_pair: &KeyPairType) -> Result<()> {
        let pre_key_id = key_id
            .as_f64()
            .ok_or(anyhow!("Invalid pre_key id {:?}", key_id))
            .map(|value| PreKeyId::from(value as u32))
            .map_err(WasmError::from)?;

        let record = PreKeyRecord {
            id: pre_key_id,
            public_key: PublicKey::from(key_pair.public_key.as_ref()),
            private_key: PrivateKey::from(key_pair.private_key.as_ref()),
        };

        self.encryption_keys_repo
            .put_pre_keys(&[record])
            .await
            .map_err(WasmError::from)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "removePreKey")]
    pub async fn remove_pre_key(&self, key_id: &JsValue) -> Result<()> {
        let pre_key_id = key_id
            .as_f64()
            .ok_or(anyhow!("Invalid pre_key id {:?}", key_id))
            .map(|value| PreKeyId::from(value as u32))
            .map_err(WasmError::from)?;

        self.encryption_keys_repo
            .delete_pre_key(pre_key_id)
            .await
            .map_err(WasmError::from)?;

        Ok(())
    }

    #[wasm_bindgen(js_name = "storeSession")]
    pub async fn store_session(&self, encoded_address: &str, record: &str) -> Result<()> {
        let (user_id, device_id) = try_decode_address(encoded_address).map_err(WasmError::from)?;
        self.session_repo
            .put_session_data(
                &user_id,
                &device_id,
                SessionData::from(record.as_bytes().to_vec().into_boxed_slice()),
            )
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "loadSession")]
    pub async fn load_session(&self, encoded_address: &str) -> Result<Option<SessionRecordType>> {
        let (user_id, device_id) = try_decode_address(encoded_address).map_err(WasmError::from)?;
        let session = self
            .session_repo
            .get_session(&user_id, &device_id)
            .await
            .map_err(WasmError::from)?
            .and_then(|session| {
                session
                    .data
                    .map(|data| String::from_utf8_lossy(data.as_ref()).into_owned())
            });
        Ok(session)
    }

    #[wasm_bindgen(js_name = "loadSignedPreKey")]
    pub async fn load_signed_pre_key(&self, key_id: &JsValue) -> Result<Option<KeyPairType>> {
        let signed_pre_key_id = key_id
            .as_f64()
            .ok_or(anyhow!("Invalid signed_pre_key id {:?}", key_id))
            .map(|value| SignedPreKeyId::from(value as u32))
            .map_err(WasmError::from)?;

        Ok(self
            .encryption_keys_repo
            .get_signed_pre_key(signed_pre_key_id.into())
            .await
            .map_err(WasmError::from)?
            .map(KeyPairType::from))
    }

    #[wasm_bindgen(js_name = "storeSignedPreKey")]
    pub async fn store_signed_pre_key(
        &self,
        record: &SignedPreKeyPairType,
        timestamp: u32,
    ) -> Result<()> {
        let record = SignedPreKeyRecord {
            id: SignedPreKeyId::from(record.key_id),
            public_key: PublicKey::from(record.key_pair.public_key.as_ref()),
            private_key: PrivateKey::from(record.key_pair.private_key.as_ref()),
            signature: record.signature.clone(),
            timestamp: timestamp as u64,
        };
        self.encryption_keys_repo
            .put_signed_pre_key(&record)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = "removeSignedPreKey")]
    pub async fn remove_signed_pre_key(&self, key_id: &JsValue) -> Result<()> {
        let signed_pre_key_id = key_id
            .as_f64()
            .ok_or(anyhow!("Invalid signed_pre_key id {:?}", key_id))
            .map(|value| SignedPreKeyId::from(value as u32))
            .map_err(WasmError::from)?;
        self.encryption_keys_repo
            .delete_signed_pre_key(signed_pre_key_id)
            .await
            .map_err(WasmError::from)?;
        Ok(())
    }
}

#[wasm_bindgen]
impl KeyPairType {
    #[wasm_bindgen(constructor)]
    pub fn new(public_key: Box<[u8]>, private_key: Box<[u8]>) -> Self {
        Self {
            public_key,
            private_key,
        }
    }

    #[wasm_bindgen(getter, js_name = "publicKey")]
    pub fn public_key(&self) -> Box<[u8]> {
        self.public_key.clone()
    }

    #[wasm_bindgen(setter, js_name = "publicKey")]
    pub fn set_public_key(&mut self, public_key: Box<[u8]>) {
        self.public_key = public_key
    }

    #[wasm_bindgen(getter, js_name = "privateKey")]
    pub fn private_key(&self) -> Box<[u8]> {
        self.private_key.clone()
    }

    #[wasm_bindgen(setter, js_name = "privateKey")]
    pub fn set_private_key(&mut self, private_key: Box<[u8]>) {
        self.private_key = private_key
    }
}

#[wasm_bindgen]
impl PreKeyPairType {
    #[wasm_bindgen(constructor)]
    pub fn new(key_id: u32, key_pair: KeyPairType) -> Self {
        Self { key_id, key_pair }
    }

    #[wasm_bindgen(getter, js_name = "keyId")]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    #[wasm_bindgen(setter, js_name = "keyId")]
    pub fn set_key_id(&mut self, key_id: u32) {
        self.key_id = key_id
    }

    #[wasm_bindgen(getter, js_name = "privateKey")]
    pub fn key_pair(&self) -> KeyPairType {
        self.key_pair.clone()
    }

    #[wasm_bindgen(setter, js_name = "privateKey")]
    pub fn set_key_pair(&mut self, key_pair: KeyPairType) {
        self.key_pair = key_pair
    }
}

#[wasm_bindgen]
impl SignedPreKeyPairType {
    #[wasm_bindgen(constructor)]
    pub fn new(key_id: u32, key_pair: KeyPairType, signature: Box<[u8]>) -> Self {
        Self {
            key_id,
            key_pair,
            signature,
        }
    }

    #[wasm_bindgen(getter, js_name = "keyId")]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    #[wasm_bindgen(setter, js_name = "keyId")]
    pub fn set_key_id(&mut self, key_id: u32) {
        self.key_id = key_id
    }

    #[wasm_bindgen(getter, js_name = "privateKey")]
    pub fn key_pair(&self) -> KeyPairType {
        self.key_pair.clone()
    }

    #[wasm_bindgen(setter, js_name = "privateKey")]
    pub fn set_key_pair(&mut self, key_pair: KeyPairType) {
        self.key_pair = key_pair
    }

    #[wasm_bindgen(getter, js_name = "signature")]
    pub fn signature(&self) -> Box<[u8]> {
        self.signature.clone()
    }

    #[wasm_bindgen(setter, js_name = "signature")]
    pub fn set_signature(&mut self, signature: Box<[u8]>) {
        self.signature = signature
    }
}

#[wasm_bindgen]
impl PreKeyType {
    #[wasm_bindgen(constructor)]
    pub fn new(key_id: u32, public_key: Box<[u8]>) -> Self {
        Self { key_id, public_key }
    }

    #[wasm_bindgen(getter, js_name = "keyId")]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    #[wasm_bindgen(setter, js_name = "keyId")]
    pub fn set_key_id(&mut self, key_id: u32) {
        self.key_id = key_id
    }

    #[wasm_bindgen(getter, js_name = "publicKey")]
    pub fn public_key(&self) -> Box<[u8]> {
        self.public_key.clone()
    }

    #[wasm_bindgen(setter, js_name = "publicKey")]
    pub fn set_public_key(&mut self, public_key: Box<[u8]>) {
        self.public_key = public_key
    }
}

#[wasm_bindgen]
impl SignedPublicPreKeyType {
    #[wasm_bindgen(constructor)]
    pub fn new(key_id: u32, public_key: Box<[u8]>, signature: Box<[u8]>) -> Self {
        Self {
            key_id,
            public_key,
            signature,
        }
    }

    #[wasm_bindgen(getter, js_name = "keyId")]
    pub fn key_id(&self) -> u32 {
        self.key_id
    }

    #[wasm_bindgen(setter, js_name = "keyId")]
    pub fn set_key_id(&mut self, key_id: u32) {
        self.key_id = key_id
    }

    #[wasm_bindgen(getter, js_name = "publicKey")]
    pub fn public_key(&self) -> Box<[u8]> {
        self.public_key.clone()
    }

    #[wasm_bindgen(setter, js_name = "publicKey")]
    pub fn set_public_key(&mut self, public_key: Box<[u8]>) {
        self.public_key = public_key
    }

    #[wasm_bindgen(getter, js_name = "signature")]
    pub fn signature(&self) -> Box<[u8]> {
        self.signature.clone()
    }

    #[wasm_bindgen(setter, js_name = "signature")]
    pub fn set_signature(&mut self, signature: Box<[u8]>) {
        self.signature = signature
    }
}

#[wasm_bindgen]
impl PreKeyBundle {
    #[wasm_bindgen(getter, js_name = "identityKey")]
    pub fn identity_key(&self) -> Box<[u8]> {
        self.identity_key.clone()
    }

    #[wasm_bindgen(getter, js_name = "signedPreKey")]
    pub fn signed_pre_key(&self) -> SignedPublicPreKeyType {
        self.signed_pre_key.clone()
    }

    #[wasm_bindgen(getter, js_name = "preKey")]
    pub fn pre_key(&self) -> PreKeyType {
        self.pre_key.clone()
    }

    #[wasm_bindgen(getter, js_name = "registrationId")]
    pub fn registration_id(&self) -> u32 {
        self.registration_id.clone()
    }
}

#[wasm_bindgen]
impl LocalEncryptionBundle {
    #[wasm_bindgen(constructor)]
    pub fn new(
        identity_key: KeyPairType,
        signed_pre_key: SignedPreKeyPairType,
        pre_keys: Vec<PreKeyPairType>,
    ) -> Self {
        Self {
            identity_key,
            signed_pre_key,
            pre_keys,
        }
    }
}

#[wasm_bindgen]
impl EncryptedMessage {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Box<[u8]>, prekey: bool) -> Self {
        Self { data, prekey }
    }
}
