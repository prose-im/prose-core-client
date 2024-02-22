// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::AtomicBool;

use anyhow::Result;
use jid::BareJid;
use parking_lot::RwLock;

use crate::domain::connection::models::{ConnectionProperties, HttpUploadService};
use crate::domain::general::models::{Capabilities, SoftwareVersion};
use crate::dtos::UserResourceId;

pub struct AppConfig {
    /// The number of messages to return in a MessageResultSet.
    pub message_page_size: u32,
    /// The maximum number of pages to fetch when trying to fill a MessageResultSet.
    pub max_message_pages_to_load: u32,
}

pub struct AppContext {
    pub connection_properties: RwLock<Option<ConnectionProperties>>,
    pub capabilities: Capabilities,
    pub software_version: SoftwareVersion,
    pub is_observing_rooms: AtomicBool,
    pub config: AppConfig,
}

impl AppContext {
    pub fn new(capabilities: Capabilities, software_version: SoftwareVersion) -> Self {
        Self {
            connection_properties: Default::default(),
            capabilities,
            software_version,
            is_observing_rooms: Default::default(),
            config: Default::default(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            message_page_size: 100,
            max_message_pages_to_load: 5,
        }
    }
}

impl AppContext {
    pub fn connected_id(&self) -> Result<UserResourceId> {
        self.connection_properties
            .read()
            .as_ref()
            .map(|p| p.connected_jid.clone())
            .ok_or(anyhow::anyhow!(
                "Failed to read the user's JID since the client is not connected."
            ))
    }

    pub fn muc_service(&self) -> Result<BareJid> {
        self.connection_properties
            .read()
            .as_ref()
            .and_then(|p| p.server_features.muc_service.clone())
            .ok_or(anyhow::anyhow!("Server does not support MUC (XEP-0045)"))
    }

    pub fn http_upload_service(&self) -> Result<HttpUploadService> {
        self.connection_properties
            .read()
            .as_ref()
            .and_then(|p| p.server_features.http_upload_service.clone())
            .ok_or(anyhow::anyhow!(
                "Server does not support HTTP uploads (XEP-0363)"
            ))
    }
}

impl AppContext {
    pub fn set_connection_properties(&self, properties: ConnectionProperties) {
        self.connection_properties.write().replace(properties);
    }

    pub fn reset_connection_properties(&self) {
        self.connection_properties.write().take();
    }
}
