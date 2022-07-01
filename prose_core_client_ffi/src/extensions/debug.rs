use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use libstrophe::Stanza;
use std::sync::Arc;

pub struct Debug {
    ctx: Arc<XMPPExtensionContext>,
}

impl Debug {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        Debug { ctx }
    }
}

impl XMPPExtension for Debug {}

impl Debug {
    pub fn send_xml_payload(&self, xml_str: &str) -> Result<()> {
        let stanza = Stanza::from_str(xml_str);
        self.ctx.send_stanza(stanza)
    }
}
