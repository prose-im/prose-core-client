// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;
use xmpp_parsers::muc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AbstractRoom {
    pub jid: BareJid,
    pub nick: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub occupants: Vec<Occupant>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Occupant {
    pub affiliation: muc::user::Affiliation,
    pub occupant_id: Option<String>,
}

impl Eq for Occupant {}
