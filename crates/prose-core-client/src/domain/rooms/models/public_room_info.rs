// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::RoomId;

#[derive(Debug, Clone, PartialEq)]
pub struct PublicRoomInfo {
    pub jid: RoomId,
    pub name: Option<String>,
}
