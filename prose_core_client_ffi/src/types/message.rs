use jid::BareJid;
use libstrophe::Stanza;
use std::convert::TryFrom;
use std::str::FromStr;

pub struct Message {
    pub from: BareJid,
    pub body: String,
}

impl TryFrom<&Stanza> for Message {
    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let from = stanza
            .from()
            .map(BareJid::from_str)
            .ok_or(())?
            .map_err(|_| ())?;
        let body = stanza
            .get_child_by_name("body")
            .and_then(|b| b.text())
            .ok_or(())?;

        Ok(Message { from, body })
    }

    type Error = ();
}
