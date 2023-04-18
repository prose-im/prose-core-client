use jid::BareJid;
use prose_core_lib::modules;

#[derive(Debug, PartialEq, Clone)]
pub struct RosterItem {
    pub jid: BareJid,
    pub subscription: modules::roster::Subscription,
    pub groups: Vec<String>,
}

impl TryFrom<&modules::roster::Item<'_>> for RosterItem {
    type Error = ();

    fn try_from(stanza: &modules::roster::Item) -> Result<Self, Self::Error> {
        let (Some(jid), sub, groups) = (stanza.jid(), stanza.subscription(), stanza.groups()) else {
            return Err(());
        };
        return Ok(RosterItem {
            jid,
            subscription: sub,
            groups,
        });
    }
}
