use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::types::mam::{MAMPreferences, Preferences};
use crate::types::namespace::Namespace;
use libstrophe::Stanza;
use std::ops::Deref;
use std::sync::Arc;

pub struct MAM {
    ctx: Arc<XMPPExtensionContext>,
}

impl MAM {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Self {
        MAM { ctx }
    }
}

impl XMPPExtension for MAM {
    fn handle_iq_stanza(&self, stanza: &Stanza) -> Result<()> {
        if let Some(prefs_node) = stanza.get_child_by_name_and_ns("prefs", Namespace::MAM2) {
            self.ctx.observer.did_receive_archiving_preferences(
                Preferences::try_from(prefs_node.deref())?.into(),
            );
        }
        Ok(())
    }
}

impl MAM {
    pub fn load_archiving_preferences(&self) -> Result<()> {
        let mut prefs = Stanza::new();
        prefs.set_name("prefs")?;
        prefs.set_ns(Namespace::MAM2)?;

        let mut iq = Stanza::new_iq(Some("get"), Some(&self.ctx.generate_id()));
        iq.add_child(prefs)?;

        self.ctx.send_stanza(iq)
    }

    pub fn set_archiving_preferences(&self, preferences: &MAMPreferences) -> Result<()> {
        let mut iq = Stanza::new_iq(Some("set"), Some(&self.ctx.generate_id()));
        iq.add_child(preferences.try_into()?)?;
        self.ctx.send_stanza(iq)
    }
}
