// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::Mutex;

use crate::domain::encryption::models::DeviceId;
use crate::domain::shared::models::UserId;
use crate::dtos::PreKeyId;

/// Collects used PreKeys. If you're creating a `DecryptionContext` make sure to call
/// `EncryptionDomainService::finalize_decryption` after use.
#[derive(Debug, Clone)]
pub struct DecryptionContext {
    inner: Arc<Mutex<DecryptionContextInner>>,
}

impl Default for DecryptionContext {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl PartialEq for DecryptionContext {
    fn eq(&self, other: &Self) -> bool {
        self.inner.lock().eq(&*other.inner.lock())
    }
}

impl DecryptionContext {
    pub fn insert_message_sender(&self, user_id: UserId, device_id: DeviceId) {
        self.inner
            .lock()
            .message_senders
            .insert((user_id, device_id));
    }

    pub fn insert_used_pre_key(&self, id: PreKeyId) {
        self.inner.lock().used_pre_keys.insert(id);
    }

    pub fn insert_broken_session(&self, user_id: UserId, device_id: DeviceId) {
        self.inner
            .lock()
            .broken_sessions
            .insert((user_id, device_id));
    }
}

impl DecryptionContext {
    pub fn into_inner(self) -> Option<DecryptionContextInner> {
        Arc::into_inner(self.inner).map(|mutex| mutex.into_inner())
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct DecryptionContextInner {
    pub message_senders: HashSet<(UserId, DeviceId)>,
    pub broken_sessions: HashSet<(UserId, DeviceId)>,
    pub used_pre_keys: HashSet<PreKeyId>,
}
