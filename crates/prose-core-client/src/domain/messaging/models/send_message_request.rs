// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct SendMessageRequest {
    pub body: Option<String>,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub url: Url,
    pub description: Option<String>,
}
