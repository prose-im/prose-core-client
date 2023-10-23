// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::BareJid;

use crate::util::StringExt;

pub trait JidExt {
    fn to_display_name(&self) -> String;
}

impl JidExt for BareJid {
    fn to_display_name(&self) -> String {
        let Some(node) = self.node_str() else {
            return self.to_string().to_uppercase_first_letter();
        };

        node.split_terminator(&['.', '_', '-'][..])
            .map(|s| s.to_uppercase_first_letter())
            .collect::<Vec<_>>()
            .join(" ")
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
