// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

/// `CapabilitiesId` represents an XMPP capabilities node identifier, concatenating a 'node' URL
/// and a 'ver' version string, separated by '#'.
/// https://xmpp.org/extensions/xep-0115.html
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilitiesId(String);

impl CapabilitiesId {
    pub fn new(node: impl AsRef<str>, ver: impl AsRef<str>) -> Self {
        Self(format!("{}#{}", node.as_ref(), ver.as_ref()))
    }
}

impl<T> From<T> for CapabilitiesId
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        CapabilitiesId(s.into())
    }
}

impl AsRef<str> for CapabilitiesId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for CapabilitiesId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}
