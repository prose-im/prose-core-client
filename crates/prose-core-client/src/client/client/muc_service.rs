use anyhow::Result;
use jid::BareJid;
use prose_xmpp::mods::{muc, MUC};
use prose_xmpp::Client as XMPPClient;

pub struct MUCService {
    pub jid: BareJid,
    pub(super) client: XMPPClient,
}

impl MUCService {
    pub async fn load_public_rooms(&self) -> Result<Vec<muc::Room>> {
        let muc = self.client.get_mod::<MUC>();
        muc.load_public_rooms(&self.jid).await
    }

    pub async fn create_instant_room(&self) -> Result<()> {
        let muc = self.client.get_mod::<MUC>();
        muc.create_instant_room(&self.jid, "new_room").await
    }
}
