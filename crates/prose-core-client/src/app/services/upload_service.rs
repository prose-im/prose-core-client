// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::path::Path;

use anyhow::{format_err, Result};
use mime_guess::mime;

use prose_proc_macros::InjectDependencies;

use crate::app::deps::{DynAppContext, DynUploadService};
use crate::dtos::UploadSlot;

#[derive(InjectDependencies)]
pub struct UploadService {
    #[inject]
    ctx: DynAppContext,
    #[inject]
    upload_service: DynUploadService,
}

impl UploadService {
    pub async fn request_upload_slot(&self, file_name: &str, file_size: u64) -> Result<UploadSlot> {
        let service = self.ctx.http_upload_service()?;

        if file_size > service.max_file_size {
            return Err(format_err!(
                "File exceeds maximum file size of upload service."
            ));
        }

        let content_type = mime_guess::from_path(file_name)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM);

        let slot = self
            .upload_service
            .request_upload_slot(&service.host, file_name, file_size, content_type.as_ref())
            .await?;

        Ok(slot)
    }

    pub async fn request_upload_slot_for_file(&self, path: impl AsRef<Path>) -> Result<UploadSlot> {
        let path = path.as_ref();
        let Some(path_str) = path.file_name().and_then(|f| f.to_str()) else {
            return Err(format_err!("Invalid filepath."));
        };
        let metadata = path.metadata()?;

        self.request_upload_slot(path_str, metadata.len()).await
    }
}
