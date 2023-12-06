// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Debug, Display, Formatter};

use jid::Jid;

#[derive(Clone, PartialEq, Eq, Hash)]
/// Represents a unspecified XMPP identifier. Could be a user, server, user resource, etc.…
pub struct SenderId(Jid);

impl SenderId {
    pub fn into_inner(self) -> Jid {
        self.0
    }
}

impl From<Jid> for SenderId {
    fn from(value: Jid) -> Self {
        SenderId(value)
    }
}

impl Debug for SenderId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SenderId({})", self.0)
    }
}

impl Display for SenderId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
