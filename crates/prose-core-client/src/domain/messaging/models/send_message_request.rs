// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::EncryptedPayload;
use crate::domain::shared::models::{Markdown, StyledMessage};

use super::{Attachment, Mention, MessageRemoteId};

#[derive(Debug, Clone, PartialEq)]
pub struct SendMessageRequest {
    pub id: MessageRemoteId,
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
    Unencrypted {
        message: Markdown,
        fallback: StyledMessage,
    },
    Encrypted(EncryptedPayload),
}
