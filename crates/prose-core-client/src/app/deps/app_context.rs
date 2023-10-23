// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::atomic::AtomicBool;

use anyhow::Result;
use jid::{BareJid, FullJid};
use parking_lot::RwLock;

use crate::domain::general::models::{Capabilities, SoftwareVersion};

pub struct AppContext {
    pub connected_jid: RwLock<Option<FullJid>>,
    pub muc_service: RwLock<Option<BareJid>>,
    pub capabilities: Capabilities,
    pub software_version: SoftwareVersion,
    pub is_observing_rooms: AtomicBool,
}

impl AppContext {
    pub fn connected_jid(&self) -> Result<FullJid> {
        self.connected_jid.read().clone().ok_or(anyhow::anyhow!(
            "Failed to read the user's JID since the client is not connected."
        ))
    }

    pub fn muc_service(&self) -> Result<BareJid> {
        self.muc_service
            .read()
            .clone()
            .ok_or(anyhow::anyhow!("Server does not support MUC (XEP-0045)"))
    }
}
