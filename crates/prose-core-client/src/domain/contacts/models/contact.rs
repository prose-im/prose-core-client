// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub jid: BareJid,
    pub name: Option<String>,
    pub group: Group,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Group {
    Favorite,
    Team,
    Other,
}
