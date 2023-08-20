// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::{mods, Client, Event, IDProvider};
use anyhow::Result;
use async_trait::async_trait;
#[cfg(feature = "test")]
pub use connector::{Connection, Connector};
pub use incrementing_id_provider::IncrementingIDProvider;
use jid::{BareJid, DomainPart, FullJid, NodePart};
use parking_lot::RwLock;
use std::str::FromStr;
use std::sync::Arc;

mod connector;
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
macro_rules! jid_str {
    ($jid:expr) => {
        $jid.parse::<jid::Jid>().unwrap()
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
