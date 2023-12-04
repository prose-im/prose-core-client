// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, FullJid, Jid};

use crate::util::StringExt;

pub trait BareJidExt {
    fn to_display_name(&self) -> String;
}

pub trait FullJidExt {
    fn resource_to_display_name(&self) -> String;
}

pub trait JidExt {
    fn node_to_display_name(&self) -> String;
    fn resource_to_display_name(&self) -> String;
}

impl BareJidExt for BareJid {
    fn to_display_name(&self) -> String {
        let Some(node) = self.node_str() else {
            return self.to_string().to_uppercase_first_letter();
        };
        node.capitalized_display_name()
    }
}

impl FullJidExt for FullJid {
    fn resource_to_display_name(&self) -> String {
        self.resource_str().capitalized_display_name()
    }
}

impl JidExt for Jid {
    fn node_to_display_name(&self) -> String {
        let Some(node) = self.node_str() else {
            return self.to_string().to_uppercase_first_letter();
        };
        node.capitalized_display_name()
    }

    fn resource_to_display_name(&self) -> String {
        let Some(resource) = self.resource_str() else {
            return self.to_string().to_uppercase_first_letter();
        };
        resource.capitalized_display_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prose_xmpp::bare;

    #[test]
    fn test_display_name() {
        assert_eq!(&bare!("abc@prose.org").to_display_name(), "Abc");
        assert_eq!(&bare!("jane-doe@prose.org").to_display_name(), "Jane Doe");
        assert_eq!(&bare!("jane.doe@prose.org").to_display_name(), "Jane Doe");
        assert_eq!(&bare!("jane_doe@prose.org").to_display_name(), "Jane Doe");
    }
}
