// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::RequestError;
use anyhow::Result;
use jid::BareJid;
use xmpp_parsers::http_upload::{SlotRequest, SlotResult};
use xmpp_parsers::iq::Iq;

// XEP-0363: HTTP File Upload
// https://xmpp.org/extensions/xep-0363.html#upload

#[derive(Default, Clone)]
pub struct HttpUpload {
    ctx: ModuleContext,
}

impl Module for HttpUpload {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl HttpUpload {
    pub async fn request_slot(
        &self,
        service: &BareJid,
        file_name: &str,
        file_size: u64,
        content_type: Option<&str>,
    ) -> Result<SlotResult> {
        let response = self
            .ctx
            .send_iq(
                Iq::from_get(
                    self.ctx.generate_id(),
                    SlotRequest {
                        filename: file_name.to_string(),
                        size: file_size,
                        content_type: content_type.map(ToString::to_string),
                    },
                )
                .with_to(service.clone().into()),
            )
            .await?;

        let Some(response) = response else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(SlotResult::try_from(response)?)
    }
}
