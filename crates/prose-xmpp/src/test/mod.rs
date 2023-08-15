// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

#[cfg(feature = "test")]
pub use connector::{Connection, Connector};

mod connector;

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
