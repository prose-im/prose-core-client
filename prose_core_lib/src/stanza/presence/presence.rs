use crate::helpers::id_string_macro::id_string;
use crate::helpers::stanza_base_macro::stanza_base;
use crate::helpers::StanzaCow;
use crate::stanza::presence::Caps;
use crate::stanza::Namespace;

use super::{Kind, Show};

id_string!(Id);

pub struct Presence<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Presence<'a> {
    pub fn new() -> Self {
        Presence {
            stanza: libstrophe::Stanza::new_presence().into(),
        }
    }
}

impl<'a> Presence<'a> {
    pub fn kind(&self) -> Option<Kind> {
        self.stanza
            .get_attribute("type")
            .and_then(|s| s.parse::<Kind>().ok())
    }

    pub fn set_kind(self, kind: Kind) -> Self {
        self.set_attribute("type", kind.to_string())
    }

    pub fn set_id(mut self, id: Id) -> Self {
        self.stanza
            .to_mut()
            .set_id(id.as_ref())
            .expect("Failed to set id");
        self
    }

    pub fn show(&self) -> Option<Show> {
        self.child_by_name("show")?
            .text()
            .and_then(|s| s.parse::<Show>().ok())
    }

    pub fn set_show(self, show: Show) -> Self {
        self.add_child(Stanza::new_text_node("show", show.to_string()))
    }

    pub fn status(&self) -> Option<String> {
        self.child_by_name("status")?.text()
    }

    pub fn set_status(self, status: impl AsRef<str>) -> Self {
        self.add_child(Stanza::new_text_node("status", status))
    }

    pub fn caps(&self) -> Option<Caps> {
        self.child_by_name_and_namespace("c", Namespace::Caps)
            .map(Into::into)
    }

    pub fn set_caps(self, caps: Caps) -> Self {
        self.add_child(caps)
    }
}

stanza_base!(Presence);

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use jid::Jid;

    use super::*;

    #[test]
    fn test_builder() {
        let iq = Presence::new()
            .set_to(Jid::from_str("a@prose.org").unwrap())
            .set_from(Jid::from_str("b@prose.org").unwrap())
            .set_show(Show::Chat)
            .set_kind(Kind::Subscribe)
            .set_status("Hello World");

        assert_eq!(
            iq.to_string(),
            r#"<presence type="subscribe" to="a@prose.org" from="b@prose.org"><show>chat</show><status>Hello World</status></presence>"#
        );
    }
}
