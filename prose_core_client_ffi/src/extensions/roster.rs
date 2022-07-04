use crate::error::Result;
use crate::extensions::{XMPPExtension, XMPPExtensionContext};
use crate::helpers::StanzaExt;
use crate::types;
use crate::types::namespace::Namespace;
use libstrophe::Stanza;
use std::sync::{Arc, Mutex};

pub struct Roster {
    ctx: Arc<XMPPExtensionContext>,
    roster: Arc<Mutex<types::roster::Roster>>,
}

impl Roster {
    pub fn new(ctx: Arc<XMPPExtensionContext>) -> Roster {
        Roster {
            ctx,
            roster: Arc::new(Mutex::new(types::roster::Roster::default())),
        }
    }
}

impl XMPPExtension for Roster {
    fn handle_iq_stanza(&self, stanza: &Stanza) -> Result<()> {
        if !stanza.has_namespace(Namespace::Roster) {
            return Ok(());
        }
        let updated_roster: types::roster::Roster = stanza.try_into()?;
        let mut roster = self.roster.lock()?;
        *roster = updated_roster;
        self.ctx.observer.did_receive_roster(roster.clone());
        Ok(())
    }
}

impl Roster {
    pub fn load_roster(&self) -> Result<()> {
        let mut iq_stanza = Stanza::new_iq(Some("get"), Some(&self.ctx.generate_id()));
        iq_stanza.add_child(Stanza::new_query(Namespace::Roster)?)?;
        self.ctx.send_stanza(iq_stanza)
    }

    // pub fn add_user(&self, nickname: Option<&str>, groups: &[&str]) -> Result<()> {
    //     let id = Uuid::new_v4().to_string();
    //     let mut iq_stanza = Stanza::new_iq(Some("set"), Some(&id));
    //     let query = Stanza::new_query(Namespace::Roster)?;
    //
    // }
}
