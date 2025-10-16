// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, Utc};
use jid::BareJid;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};

use crate::domain::connection::models::{ConnectionProperties, HttpUploadService, ServerFeatures};
use crate::domain::general::models::{Capabilities, SoftwareVersion};
use crate::domain::shared::models::{AccountId, ConnectionState};
use crate::dtos::{DecryptionContext, MucId, UserResourceId};

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The number of messages to return in a MessageResultSet.
    pub message_page_size: u32,
    /// The maximum number of pages to fetch when trying to fill a MessageResultSet.
    pub max_message_pages_to_load: u32,
    /// The maximum duration to fetch messages into the past during catchup.
    pub max_catchup_duration_secs: i64,
}

pub struct AppContext {
    pub connection_properties: RwLock<Option<ConnectionProperties>>,
    pub connection_state: RwLock<ConnectionState>,
    pub capabilities: Capabilities,
    pub software_version: SoftwareVersion,
    pub config: AppConfig,
}

impl AppContext {
    pub fn new(
        capabilities: Capabilities,
        software_version: SoftwareVersion,
        config: AppConfig,
    ) -> Self {
        Self {
            connection_properties: Default::default(),
            connection_state: Default::default(),
            capabilities,
            software_version,
            config,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            message_page_size: 100,
            max_message_pages_to_load: 5,
            max_catchup_duration_secs: 60 * 60 * 24 * 5,
        }
    }
}

impl AppContext {
    pub fn connection_state(&self) -> ConnectionState {
        *self.connection_state.read()
    }

    pub fn connected_id(&self) -> Result<UserResourceId> {
        self.connection_properties
            .read()
            .as_ref()
            .map(|p| p.connected_jid.clone())
            .ok_or(anyhow::anyhow!(
                "Failed to read the user's JID since the client is not connected."
            ))
    }

    pub fn connected_account(&self) -> Result<AccountId> {
        Ok(AccountId::from(
            self.connected_id()?.into_inner().into_bare(),
        ))
    }

    pub fn connection_timestamp(&self) -> Result<DateTime<Utc>> {
        self.connection_properties
            .read()
            .as_ref()
            .map(|p| p.connection_timestamp.clone())
            .ok_or(anyhow::anyhow!(
                "Failed to read the connection timestamp since the client is not connected."
            ))
    }

    pub fn muc_service(&self) -> Result<BareJid> {
        self.server_features()?
            .muc_service
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server does not support MUC (XEP-0045)"))
            .cloned()
    }

    pub fn http_upload_service(&self) -> Result<HttpUploadService> {
        self.server_features()?
            .http_upload_service
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Server does not support HTTP uploads (XEP-0363)"))
            .cloned()
    }

    pub fn server_features<'a>(&'a self) -> Result<MappedRwLockReadGuard<'a, ServerFeatures>> {
        let read_guard = self.connection_properties.read();
        if read_guard.is_none() {
            return Err(anyhow::anyhow!(
                "Failed to read the server features since the client is not connected."
            ));
        }
        return Ok(RwLockReadGuard::map(
            self.connection_properties.read(),
            // We can safely unwrap the Option here since we've checked that it's not empty above.
            |s| &s.as_ref().unwrap().server_features,
        ));
    }

    /// Have we loaded the unread messages for the rooms in our sidebar?
    pub fn rooms_caught_up(&self) -> bool {
        self.connection_properties
            .read()
            .as_ref()
            .map(|p| p.rooms_caught_up)
            .unwrap_or_default()
    }

    pub fn decryption_context(&self) -> Option<DecryptionContext> {
        self.connection_properties
            .read()
            .as_ref()
            .and_then(|p| p.decryption_context.clone())
    }
}

impl AppContext {
    pub fn is_muc_room_on_connected_server(&self, room_id: &MucId) -> bool {
        let props = self.connection_properties.read();
        let muc_service_id = BareJid::from_parts(None, &room_id.as_ref().domain());
        Some(&muc_service_id)
            == props
                .as_ref()
                .and_then(|p| p.server_features.muc_service.as_ref())
    }
}

impl AppContext {
    pub fn set_connection_properties(&self, properties: ConnectionProperties) {
        self.connection_properties.write().replace(properties);
    }

    pub fn reset_connection_properties(&self) {
        self.connection_properties.write().take();
    }

    pub fn set_connection_state(&self, state: ConnectionState) {
        *self.connection_state.write() = state;
    }

    pub fn set_rooms_caught_up(&self) {
        self.connection_properties
            .write()
            .as_mut()
            .map(|p| p.rooms_caught_up = true);
    }

    pub fn take_decryption_context(&self) -> Option<DecryptionContext> {
        self.connection_properties
            .write()
            .as_mut()
            .and_then(|p| p.decryption_context.take())
    }
}
