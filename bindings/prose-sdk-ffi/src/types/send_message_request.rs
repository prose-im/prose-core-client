// prose-core-client/prose-sdk-ffi
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::types::Attachment;
use prose_core_client::dtos::{
    SendMessageRequest as CoreSendMessageRequest,
    SendMessageRequestBody as CoreSendMessageRequestBody,
};

#[derive(uniffi::Record)]
pub struct SendMessageRequest {
    pub body: Option<SendMessageRequestBody>,
    pub attachments: Vec<Attachment>,
}

#[derive(uniffi::Record)]
pub struct SendMessageRequestBody {
    pub text: String,
}

impl From<SendMessageRequest> for CoreSendMessageRequest {
    fn from(value: SendMessageRequest) -> Self {
        CoreSendMessageRequest {
            body: value.body.map(Into::into),
            attachments: value.attachments.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SendMessageRequestBody> for CoreSendMessageRequestBody {
    fn from(value: SendMessageRequestBody) -> Self {
        CoreSendMessageRequestBody {
            text: value.text.into(),
        }
    }
}
