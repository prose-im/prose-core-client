use core::str::FromStr;

use wasm_bindgen::prelude::*;

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "JID")]
pub struct Jid {
    #[wasm_bindgen(skip)]
    pub bare: Option<BareJid>,
    #[wasm_bindgen(skip)]
    pub full: Option<FullJid>,
}

#[wasm_bindgen(js_class = "JID")]
impl Jid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<Jid, JsError> {
        let jid = jid::Jid::from_str(str)?;
        Ok(jid.into())
    }

    pub fn bare(&self) -> BareJid {
        if let Some(bare) = &self.bare {
            return bare.clone();
        };
        if let Some(full) = &self.full {
            return full.to_bare_jid();
        };
        unreachable!()
    }

    pub fn full(&self) -> Option<FullJid> {
        self.full.clone()
    }

    #[wasm_bindgen(js_name = "withBare")]
    pub fn with_bare(jid: &BareJid) -> Self {
        Jid {
            bare: Some(jid.clone()),
            full: None,
        }
    }

    #[wasm_bindgen(js_name = "withFull")]
    pub fn with_full(jid: &FullJid) -> Self {
        Jid {
            bare: None,
            full: Some(jid.clone()),
        }
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        let jid: jid::Jid = self.clone().into();
        jid.to_string()
    }

    pub fn equals(&self, other: &Jid) -> bool {
        self == other
    }
}

impl From<Jid> for jid::Jid {
    fn from(value: Jid) -> Self {
        if let Some(jid) = value.bare {
            return jid::Jid::Bare(jid.into());
        };
        if let Some(jid) = value.full {
            return jid::Jid::Full(jid.into());
        }
        unreachable!()
    }
}

impl From<jid::Jid> for Jid {
    fn from(value: jid::Jid) -> Self {
        match value {
            jid::Jid::Bare(jid) => Jid {
                bare: Some(jid.into()),
                full: None,
            },
            jid::Jid::Full(jid) => Jid {
                bare: None,
                full: Some(jid.into()),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "BareJID")]
pub struct BareJid {
    #[wasm_bindgen(skip)]
    pub node: Option<String>,

    #[wasm_bindgen(skip)]
    pub domain: String,
}

#[wasm_bindgen(js_class = "BareJID")]
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

#[derive(Debug, PartialEq, Clone)]
#[wasm_bindgen(js_name = "FullJID")]
pub struct FullJid {
    #[wasm_bindgen(skip)]
    pub node: Option<String>,
    #[wasm_bindgen(skip)]
    pub domain: String,
    #[wasm_bindgen(skip)]
    pub resource: String,
}

#[wasm_bindgen(js_class = "FullJID")]
impl FullJid {
    #[wasm_bindgen(constructor)]
    pub fn new(str: &str) -> Result<FullJid, JsError> {
        let full_jid = jid::FullJid::from_str(str)?;
        Ok(full_jid.into())
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

    /// The resource of the Jabber ID.
    #[wasm_bindgen(getter)]
    pub fn resource(&self) -> String {
        self.resource.clone()
    }

    #[wasm_bindgen(js_name = "toString")]
    pub fn to_string(&self) -> String {
        let jid: jid::FullJid = self.clone().into();
        jid.to_string()
    }

    #[wasm_bindgen(js_name = "bare")]
    pub fn to_bare_jid(&self) -> BareJid {
        BareJid {
            node: self.node.clone(),
            domain: self.domain.clone(),
        }
    }

    pub fn equals(&self, other: &FullJid) -> bool {
        self == other
    }
}

impl From<jid::BareJid> for BareJid {
    fn from(value: jid::BareJid) -> Self {
        BareJid {
            node: value.node,
            domain: value.domain,
        }
    }
}

impl From<BareJid> for jid::BareJid {
    fn from(value: BareJid) -> Self {
        jid::BareJid {
            node: value.node,
            domain: value.domain,
        }
    }
}

impl From<jid::FullJid> for FullJid {
    fn from(value: jid::FullJid) -> Self {
        FullJid {
            node: value.node,
            domain: value.domain,
            resource: value.resource,
        }
    }
}

impl From<FullJid> for jid::FullJid {
    fn from(value: FullJid) -> Self {
        jid::FullJid {
            node: value.node,
            domain: value.domain,
            resource: value.resource,
        }
    }
}
