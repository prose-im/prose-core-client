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

impl SendMessageRequest {
    pub fn is_empty(&self) -> bool {
        self.body
            .as_ref()
            .map(|body| body.text.is_empty())
            .unwrap_or(true)
            && self.attachments.is_empty()
    }
}
