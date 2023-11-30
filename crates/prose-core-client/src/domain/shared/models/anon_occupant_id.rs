// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents an anonymous identifier of a user within a Multi-User Chat (MUC) room.
/// See: https://xmpp.org/extensions/xep-0421.html
pub struct AnonOccupantId(String);

impl From<String> for AnonOccupantId {
    fn from(value: String) -> Self {
        AnonOccupantId(value)
    }
}

impl Display for AnonOccupantId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
