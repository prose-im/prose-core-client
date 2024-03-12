// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::{Attachment, Mention};

#[derive(Debug, Clone, PartialEq)]
pub struct SendMessageRequest {
    pub body: Option<Body>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub text: String,
    pub mentions: Vec<Mention>,
}
