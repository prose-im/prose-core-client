// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::MucId;

#[derive(Debug, Clone, PartialEq)]
pub struct PublicRoomInfo {
    pub id: MucId,
    pub name: Option<String>,
}
