// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

/// This is the combination "{node}#{ver}" of a "<c/>" element.
/// https://xmpp.org/extensions/xep-0115.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilitiesId(String);

impl<T> From<T> for CapabilitiesId
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        CapabilitiesId(s.into())
    }
}

impl Display for CapabilitiesId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
