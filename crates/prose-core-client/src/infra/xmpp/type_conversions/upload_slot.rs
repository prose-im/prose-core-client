// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::http_upload::{Header, SlotResult};

use crate::domain::uploads::models::{UploadHeader, UploadSlot};

impl TryFrom<SlotResult> for UploadSlot {
    type Error = anyhow::Error;

    fn try_from(value: SlotResult) -> Result<Self, Self::Error> {
        Ok(Self {
            upload_url: value.put.url.parse()?,
            upload_headers: value
                .put
                .headers
                .into_iter()
                .map(|h| match h {
                    Header::Authorization(value) => UploadHeader::new("authorization", value),
                    Header::Cookie(value) => UploadHeader::new("cookie", value),
                    Header::Expires(value) => UploadHeader::new("expires", value),
                })
                .collect(),
            download_url: value.get.url.parse()?,
        })
    }
}
