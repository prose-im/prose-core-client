// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::encryption::models::DeviceId;

use super::{Attachment, Mention, MessageId};

#[derive(Debug, Clone, PartialEq)]
pub struct SendMessageRequest {
    pub id: MessageId,
    pub body: Option<Body>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub payload: Payload,
    pub mentions: Vec<Mention>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    Plaintext(String),
    Encrypted(EncryptedPayload),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncryptedPayload {
    pub device_id: DeviceId,
    pub iv: Box<[u8]>,
    pub messages: Vec<EncryptedMessage>,
    pub payload: Box<[u8]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EncryptedMessage {
    /// The DeviceId the message is encrypted for
    pub device_id: DeviceId,
    pub prekey: bool,
    pub data: Box<[u8]>,
}
