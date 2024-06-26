// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::dtos::DeviceId;

#[derive(Debug, Clone, PartialEq)]
pub enum EncryptedMessage {
    Message(EncryptedPayload),
    KeyTransport(KeyTransportPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncryptedPayload {
    /// The device id of the sender
    pub device_id: DeviceId,

    /// IV used for payload encryption
    pub iv: Box<[u8]>,

    /// The key that the payload message is encrypted with, separately
    /// encrypted for each recipient device.
    pub keys: Vec<EncryptionKey>,

    /// The encrypted message body, unless empty when the message is used for a key exchange.
    pub payload: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KeyTransportPayload {
    /// The device id of the sender
    pub device_id: DeviceId,

    /// IV used for payload encryption
    pub iv: Box<[u8]>,

    /// The key that the payload message is encrypted with, separately
    /// encrypted for each recipient device.
    pub keys: Vec<EncryptionKey>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncryptionKey {
    /// The device id this key is encrypted for.
    pub device_id: DeviceId,

    /// The key element MUST be tagged with a prekey attribute set to true
    /// if a PreKeySignalMessage is being used.
    pub is_pre_key: bool,

    /// The 16 bytes key and the GCM authentication tag concatenated together
    /// and encrypted using the corresponding long-standing SignalProtocol
    /// session
    pub data: Box<[u8]>,
}

impl EncryptedPayload {
    pub fn get_key(&self, device_id: &DeviceId) -> Option<&EncryptionKey> {
        self.keys.iter().find(|key| &key.device_id == device_id)
    }
}

impl KeyTransportPayload {
    pub fn get_key(&self, device_id: &DeviceId) -> Option<&EncryptionKey> {
        self.keys.iter().find(|key| &key.device_id == device_id)
    }
}
