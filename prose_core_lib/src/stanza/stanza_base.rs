use std::str::FromStr;

use jid::Jid;

use crate::helpers::StanzaIterator;

use super::Namespace;
use super::Stanza;

pub trait StanzaBase: ToString + FromStr + Sized {
    fn stanza(&self) -> &libstrophe::Stanza;
    fn stanza_mut(&mut self) -> &mut libstrophe::Stanza;
    fn stanza_owned(self) -> libstrophe::Stanza;

    fn name(&self) -> Option<&str> {
        self.stanza().name()
    }

    fn text(&self) -> Option<String> {
        self.stanza().text()
    }

    fn set_attribute<T: AsRef<str>>(
        mut self,
        name: impl AsRef<str>,
        value: impl Into<Option<T>>,
    ) -> Self {
        let value: Option<T> = value.into();
        if let Some(value) = value {
            self.stanza_mut()
                .set_attribute(name, value)
                .expect("Failed to set attribute");
        }
        self
    }

    fn attribute(&self, name: impl AsRef<str>) -> Option<&str> {
        self.stanza().get_attribute(name)
    }

    fn set_namespace(mut self, namespace: Namespace) -> Self {
        self.stanza_mut().set_ns(namespace.to_string()).unwrap();
        self
    }

    fn namespace(&self) -> Option<Namespace> {
        self.stanza().ns().and_then(|s| s.parse::<Namespace>().ok())
    }

    fn set_from(self, from: impl Into<Jid>) -> Self {
        self.set_attribute("from", from.into().to_string())
    }

    fn from(&self) -> Option<Jid> {
        self.stanza()
            .get_attribute("from")
            .and_then(|s| s.parse::<Jid>().ok())
    }

    fn set_to(self, to: impl Into<Jid>) -> Self {
        self.set_attribute("to", to.into().to_string())
    }

    fn to(&self) -> Option<Jid> {
        self.stanza()
            .get_attribute("to")
            .and_then(|s| s.parse::<Jid>().ok())
    }

    fn add_child(mut self, stanza: impl StanzaBase) -> Self {
        self.stanza_mut()
            .add_child(stanza.stanza_owned())
            .expect("Failed to add child");
        self
    }

    fn add_children<T: StanzaBase>(mut self, stanzas: impl IntoIterator<Item = T>) -> Self {
        for stanza in stanzas {
            self.stanza_mut()
                .add_child(stanza.stanza_owned())
                .expect("Failed to add children");
        }
        self
    }

    fn first_child(&self) -> Option<Stanza> {
        self.stanza().get_first_child().map(|c| c.into())
    }

    fn child_by_name(&self, name: impl AsRef<str>) -> Option<Stanza> {
        self.stanza().get_child_by_name(name).map(|c| c.into())
    }

    fn child_by_namespace(&self, namespace: Namespace) -> Option<Stanza> {
        self.stanza()
            .get_child_by_ns(namespace.to_string())
            .map(|c| c.into())
    }

    fn child_by_name_and_namespace(
        &self,
        name: impl AsRef<str>,
        namespace: Namespace,
    ) -> Option<Stanza> {
        self.stanza()
            .get_child_by_name_and_ns(name, namespace.to_string())
            .map(|c| c.into())
    }

    fn self_or_immediate_child_has_namespace(&self, ns: Namespace) -> bool {
        let stanza = self.stanza();
        let ns = ns.to_string();

        if stanza.ns() == Some(&ns) {
            return true;
        }
        for child in stanza.children() {
            if child.ns() == Some(&ns) {
                return true;
            }
        }
        false
    }

    fn next_sibling(&self) -> Option<Stanza> {
        self.stanza().get_next().map(|s| s.into())
    }

    fn children(&self) -> StanzaIterator {
        StanzaIterator::new(self.stanza())
    }
}
