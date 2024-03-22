// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::Path;

use anyhow::{format_err, Result};
use mime::Mime;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynUploadService};
use crate::dtos::UploadSlot;
use crate::util::PathExt;

#[derive(InjectDependencies)]
pub struct UploadService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    upload_service: DynUploadService,
}

impl UploadService {
    pub async fn request_upload_slot(
        &self,
        file_name: &str,
        file_size: u64,
        media_type: Option<Mime>,
    ) -> Result<UploadSlot> {
        let service = self.ctx.http_upload_service()?;

        if file_size > service.max_file_size {
            return Err(format_err!(
                "File exceeds maximum file size of upload service."
            ));
        }

        let media_type = media_type.unwrap_or_else(|| Path::new(file_name).media_type());

        let slot = self
            .upload_service
            .request_upload_slot(&service.host, file_name, file_size, &media_type)
            .await?;

        Ok(UploadSlot {
            upload_url: slot.upload_url,
            upload_headers: slot.upload_headers,
            download_url: slot.download_url,
            file_name: file_name.to_string(),
            media_type,
            file_size,
        })
    }
}
