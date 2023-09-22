// prose-core-client/prose-sdk-js
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use core::fmt::{Debug, Display, Formatter};
use core::str::FromStr;

use wasm_bindgen::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "JID")]
pub struct BareJid(jid::BareJid);

#[wasm_bindgen(js_class = "JID")]
impl BareJid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<BareJid, JsError> {
        Ok(Self(jid::BareJid::from_str(str)?))
    }

    /// The node part of the Jabber ID, if it exists, else None.
    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Option<String> {
        self.0.node_str().map(ToString::to_string)
    }

    /// The domain of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn domain(&self) -> String {
        self.0.domain_str().to_string()
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn equals(&self, other: &BareJid) -> bool {
        self == other
    }
}

impl BareJid {
    pub fn to_full_jid_with_resource(&self, resource: &jid::ResourcePart) -> jid::FullJid {
        jid::FullJid::from_parts(self.0.node().as_ref(), &self.0.domain(), resource)
    }
}

impl From<jid::BareJid> for BareJid {
    fn from(value: jid::BareJid) -> Self {
        Self(value)
    }
}

impl From<&jid::BareJid> for BareJid {
    fn from(value: &jid::BareJid) -> Self {
        Self(value.clone())
    }
}

impl From<&BareJid> for jid::BareJid {
    fn from(value: &BareJid) -> Self {
        value.0.clone()
    }
}

impl From<BareJid> for jid::BareJid {
    fn from(value: BareJid) -> Self {
        value.0
    }
}

impl From<&BareJid> for jid::Jid {
    fn from(value: &BareJid) -> Self {
        jid::Jid::Bare(value.0.clone())
    }
}

impl AsRef<jid::BareJid> for BareJid {
    fn as_ref(&self) -> &jid::BareJid {
        &self.0
    }
}

impl Display for BareJid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
