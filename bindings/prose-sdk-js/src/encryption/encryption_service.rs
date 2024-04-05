// prose-core-client/prose-sdk-js
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::time::SystemTime;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::encryption::signal_repo::PreKeyPairType;
use prose_core_client::dtos::{
    DeviceId, EncryptedMessage, LocalEncryptionBundle, PreKeyBundle, PreKeyId, PreKeyRecord,
    SignedPreKeyRecord, UserId,
};
use prose_core_client::{DynEncryptionKeysRepository, EncryptionService as EncryptionServiceTrait};

use super::{
    EncryptedMessage as JsEncryptedMessage, LocalEncryptionBundle as JsLocalEncryptionBundle,
    PreKeyBundle as JsPreKeyBundle, SignalRepo,
};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export interface ProseEncryptionService {
    async generateLocalEncryptionBundle(): Promise<LocalEncryptionBundle>
    
    async processPreKeyBundle(
        repo: SignalRepo,
        user_id: string,
        device_id: number,
        bundle: PreKeyBundle
    ): Promise<void>
    
    async decryptKey(
        repo: SignalRepo,
        user_id: string,
        device_id: number,
        encryptedMessage: Uint8Array,
        isPreKey: boolean
    ): Promise<UInt8Array>
    
    async encryptKey(
        repo: SignalRepo,
        user_id: string,
        device_id: number,
        message: Uint8Array
    ): Promise<EncryptedMessage>
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ProseEncryptionService")]
    pub type JsEncryptionService;

    #[wasm_bindgen(method, catch, js_name = "generateLocalEncryptionBundle")]
    fn generate_local_encryption_bundle(this: &JsEncryptionService) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "generatePreKeysWithIds")]
    fn generate_pre_keys_with_ids(
        this: &JsEncryptionService,
        ids: Vec<u32>,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "processPreKeyBundle")]
    fn process_pre_key_bundle(
        this: &JsEncryptionService,
        repo: SignalRepo,
        user_id: String,
        device_id: u32,
        bundle: JsPreKeyBundle,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "decryptKey")]
    fn decrypt_key(
        this: &JsEncryptionService,
        repo: SignalRepo,
        user_id: String,
        device_id: u32,
        encrypted_message: Box<[u8]>,
        is_pre_key: bool,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, catch, js_name = "encryptKey")]
    fn encrypt_key(
        this: &JsEncryptionService,
        repo: SignalRepo,
        user_id: String,
        device_id: u32,
        message: Box<[u8]>,
    ) -> Result<JsValue, JsValue>;
}

pub struct EncryptionService {
    inner: JsEncryptionService,
    repo: DynEncryptionKeysRepository,
}

impl EncryptionService {
    pub fn new(inner: JsEncryptionService, repo: DynEncryptionKeysRepository) -> Self {
        Self { inner, repo }
    }
}

#[async_trait(? Send)]
impl EncryptionServiceTrait for EncryptionService {
    async fn generate_local_encryption_bundle(
        &self,
        device_id: DeviceId,
    ) -> Result<LocalEncryptionBundle> {
        let bundle = JsLocalEncryptionBundle::try_from(
            &await_promise(self.inner.generate_local_encryption_bundle()).await?,
        )
        .map_err(|err| anyhow!("{err}"))?;

        let mut signed_pre_key = SignedPreKeyRecord::from(bundle.signed_pre_key);
        signed_pre_key.timestamp = Utc::now().timestamp() as u64;

        Ok(LocalEncryptionBundle {
            device_id,
            identity_key_pair: bundle.identity_key.into(),
            signed_pre_key,
            pre_keys: bundle
                .pre_keys
                .into_iter()
                .map(PreKeyRecord::from)
                .collect(),
        })
    }

    async fn generate_pre_keys_with_ids(&self, ids: Vec<PreKeyId>) -> Result<Vec<PreKeyRecord>> {
        let pre_keys =
            Array::from(
                &await_promise(self.inner.generate_pre_keys_with_ids(
                    ids.into_iter().map(|id| id.into_inner()).collect(),
                ))
                .await?,
            )
            .into_iter()
            .map(|value| PreKeyPairType::try_from(&value).map(PreKeyRecord::from))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| anyhow!("{err}"))?;
        Ok(pre_keys)
    }

    async fn process_pre_key_bundle(&self, user_id: &UserId, bundle: PreKeyBundle) -> Result<()> {
        await_promise(self.inner.process_pre_key_bundle(
            SignalRepo::new(self.repo.clone()),
            user_id.to_string(),
            *bundle.device_id.as_ref(),
            JsPreKeyBundle::from(bundle),
        ))
        .await?;
        Ok(())
    }

    async fn encrypt_key(
        &self,
        recipient_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        _now: &SystemTime,
    ) -> Result<EncryptedMessage> {
        let value = JsEncryptedMessage::try_from(
            &await_promise(self.inner.encrypt_key(
                SignalRepo::new(self.repo.clone()),
                recipient_id.to_string(),
                *device_id.as_ref(),
                message.into(),
            ))
            .await?,
        )
        .map_err(|err| anyhow!("{err}"))?;

        Ok(EncryptedMessage {
            device_id: device_id.clone(),
            prekey: value.prekey,
            data: value.data,
        })
    }

    async fn decrypt_key(
        &self,
        sender_id: &UserId,
        device_id: &DeviceId,
        message: &[u8],
        is_pre_key: bool,
    ) -> Result<Box<[u8]>> {
        let value = Uint8Array::from(
            await_promise(self.inner.decrypt_key(
                SignalRepo::new(self.repo.clone()),
                sender_id.to_string(),
                *device_id.as_ref(),
                message.into(),
                is_pre_key,
            ))
            .await?,
        );
        Ok(value.to_vec().into_boxed_slice())
    }
}

async fn await_promise(promise: Result<JsValue, JsValue>) -> Result<JsValue> {
    let promise = js_sys::Promise::from(promise.map_err(js_error_to_anyhow)?);
    let future = wasm_bindgen_futures::JsFuture::from(promise);
    let js_value = future.await.map_err(js_error_to_anyhow)?;
    Ok(js_value)
}

fn js_error_to_anyhow(error: JsValue) -> anyhow::Error {
    anyhow!("JsEncryptionService threw an error: {:?}", error)
}
