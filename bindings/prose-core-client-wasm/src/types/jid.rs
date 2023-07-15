use core::str::FromStr;

use jid;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;
use wasm_bindgen_derive::TryFromJsValue;

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(namespace, into_wasm_abi, from_wasm_abi)]
pub enum Jid {
    Bare(BareJid),
    Full(FullJid),
}

#[wasm_bindgen(js_name = "jidToString")]
pub fn jid_to_string(jid: Jid) -> String {
    match jid {
        Jid::Bare(jid) => jid.to_string(),
        Jid::Full(jid) => jid.to_string(),
    }
}

#[wasm_bindgen(js_name = "parseJid")]
pub fn parse_jid(str: &str) -> Result<Jid, JsError> {
    match jid::Jid::from_str(str).unwrap() {
        jid::Jid::Bare(jid) => Ok(Jid::Bare(jid.into())),
        jid::Jid::Full(jid) => Ok(Jid::Full(jid.into())),
    }
}

#[derive(TryFromJsValue, Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct BareJid(jid::BareJid);

#[wasm_bindgen]
impl BareJid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<BareJid, JsError> {
        Ok(BareJid(jid::BareJid::from_str(str)?))
    }

    /// The node part of the Jabber ID, if it exists, else None.
    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Option<String> {
        self.0.node.clone()
    }

    /// The domain of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn domain(&self) -> String {
        self.0.domain.clone()
    }

    #[wasm_bindgen]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn equals(&self, other: &BareJid) -> bool {
        self.0 == other.0
    }
}

#[derive(TryFromJsValue, Serialize, Deserialize, Clone)]
#[wasm_bindgen]
pub struct FullJid(jid::FullJid);

#[wasm_bindgen]
impl FullJid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<FullJid, JsError> {
        Ok(FullJid(jid::FullJid::from_str(str)?))
    }

    /// The node part of the Jabber ID, if it exists, else None.
    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Option<String> {
        self.0.node.clone()
    }

    /// The domain of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn domain(&self) -> String {
        self.0.domain.clone()
    }

    /// The resource of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn resource(&self) -> String {
        self.0.resource.clone()
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    #[wasm_bindgen(js_name = "bare")]
    pub fn to_bare_jid(&self) -> BareJid {
        BareJid(self.0.clone().into())
    }

    pub fn equals(&self, other: &FullJid) -> bool {
        self.0 == other.0
    }
}

impl From<Jid> for jid::Jid {
    fn from(value: Jid) -> Self {
        match value {
            Jid::Bare(jid) => jid::Jid::Bare(jid.0),
            Jid::Full(jid) => jid::Jid::Full(jid.0),
        }
    }
}

impl From<jid::BareJid> for BareJid {
    fn from(value: jid::BareJid) -> Self {
        BareJid(value)
    }
}

impl From<BareJid> for jid::BareJid {
    fn from(value: BareJid) -> Self {
        value.0
    }
}

impl From<jid::FullJid> for FullJid {
    fn from(value: jid::FullJid) -> Self {
        FullJid(value)
    }
}

impl From<FullJid> for jid::FullJid {
    fn from(value: FullJid) -> Self {
        value.0
    }
}
