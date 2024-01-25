// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::FullJid;
use minidom::IntoAttributeValue;
use serde::{Deserialize, Serialize};

use crate::dtos::RoomId;
use crate::util::StringExt;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Represents the identifier of a user within a Multi-User Chat (MUC) room, combining the
/// room's JID with the user's unique nickname in that room.
pub struct OccupantId(FullJid);

impl OccupantId {
    pub fn nickname(&self) -> &str {
        self.0.resource_str()
    }

    pub fn formatted_nickname(&self) -> String {
        self.0.resource_str().capitalized_display_name()
    }

    pub fn room_id(&self) -> RoomId {
        self.0.to_bare().into()
    }

    pub fn into_inner(self) -> FullJid {
        self.0
    }
}

impl From<FullJid> for OccupantId {
    fn from(value: FullJid) -> Self {
        OccupantId(value)
    }
}

impl Debug for OccupantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OccupantId({})", self.0)
    }
}

impl Display for OccupantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl IntoAttributeValue for OccupantId {
    fn into_attribute_value(self) -> Option<String> {
        self.0.into_attribute_value()
    }
}

impl AsRef<FullJid> for OccupantId {
    fn as_ref(&self) -> &FullJid {
        &self.0
    }
}
