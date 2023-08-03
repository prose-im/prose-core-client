use core::str::FromStr;
use jid::{DomainPart, NodePart};

use wasm_bindgen::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "JID")]
pub struct BareJid {
    #[wasm_bindgen(skip)]
    pub node: Option<String>,

    #[wasm_bindgen(skip)]
    pub domain: String,
}

#[wasm_bindgen(js_class = "JID")]
impl BareJid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<BareJid, JsError> {
        let bare_jid = jid::BareJid::from_str(str)?;
        Ok(bare_jid.into())
    }

    /// The node part of the Jabber ID, if it exists, else None.
    #[wasm_bindgen(getter)]
    pub fn node(&self) -> Option<String> {
        self.node.clone()
    }

    /// The domain of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn domain(&self) -> String {
        self.domain.clone()
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        let jid: jid::BareJid = self.clone().into();
        jid.to_string()
    }

    pub fn equals(&self, other: &BareJid) -> bool {
        self == other
    }
}

impl From<jid::BareJid> for BareJid {
    fn from(value: jid::BareJid) -> Self {
        BareJid {
            node: value.node().map(|s| s.to_string()),
            domain: value.domain().to_string(),
        }
    }
}

impl From<BareJid> for jid::BareJid {
    fn from(value: BareJid) -> Self {
        jid::BareJid::from_parts(
            value
                .node
                .map(|node| NodePart::new(&node).unwrap())
                .as_ref(),
            &DomainPart::new(&value.domain).unwrap(),
        )
    }
}
