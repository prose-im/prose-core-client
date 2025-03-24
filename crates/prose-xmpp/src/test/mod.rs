// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, DomainPart, NodePart};

#[cfg(not(target_arch = "wasm32"))]
pub use connected_client::{ClientTestAdditions, ConnectedClient};
#[cfg(not(target_arch = "wasm32"))]
pub use connector::{Connection, Connector};
pub use constant_id_provider::ConstantIDProvider;
pub use element_ext::ElementExt;
pub use incrementing_id_provider::IncrementingIDProvider;

#[cfg(not(target_arch = "wasm32"))]
mod connected_client;
#[cfg(not(target_arch = "wasm32"))]
mod connector;
mod constant_id_provider;
mod element_ext;
mod incrementing_id_provider;

pub trait StrExt {
    fn to_xml_result_string(&self) -> String;
}

impl<T> StrExt for T
where
    T: AsRef<str>,
{
    fn to_xml_result_string(&self) -> String {
        let mut result = self.as_ref().to_string();
        result.retain(|c| c != '\n' && c != '\t');
        result.replace("  ", "")
    }
}

#[macro_export]
macro_rules! jid {
    ($jid:expr) => {
        $jid.parse::<jid::Jid>().unwrap()
    };
}

#[macro_export]
macro_rules! bare {
    ($jid:expr) => {
        $jid.parse::<jid::BareJid>().unwrap()
    };
}

#[macro_export]
macro_rules! full {
    ($jid:expr) => {
        $jid.parse::<jid::FullJid>().unwrap()
    };
}

pub trait BareJidTestAdditions {
    fn ours() -> BareJid;
    fn theirs() -> BareJid;
}

impl BareJidTestAdditions for BareJid {
    fn ours() -> BareJid {
        BareJid::from_parts(
            Some(&NodePart::new("test").unwrap()),
            &DomainPart::new("prose.org").unwrap(),
        )
    }

    fn theirs() -> BareJid {
        BareJid::from_parts(
            Some(&NodePart::new("friend").unwrap()),
            &DomainPart::new("prose.org").unwrap(),
        )
    }
}
