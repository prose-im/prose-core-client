use crate::error::Result;
use libstrophe::Stanza;

pub(crate) trait StanzaExt {
    /// The ns filter matches the namespace ('xmlns' attribute) of either the top level stanza or
    /// any of it's immediate children (this allows you do handle specific <iq/> stanzas based on
    /// the <query/> child namespace.
    fn has_namespace(&self, ns: impl AsRef<str>) -> bool;

    fn new_query(ns: impl AsRef<str>) -> Result<Stanza>;
    fn new_text_node(text: impl AsRef<str>) -> Result<Stanza>;
}

impl StanzaExt for Stanza {
    fn has_namespace(&self, ns: impl AsRef<str>) -> bool {
        if self.ns() == Some(ns.as_ref()) {
            return true;
        }
        for child in self.children() {
            if child.ns() == Some(ns.as_ref()) {
                return true;
            }
        }
        false
    }

    fn new_query(ns: impl AsRef<str>) -> Result<Self> {
        let mut query = Stanza::new();
        query.set_name("query")?;
        query.set_ns(ns)?;
        Ok(query)
    }

    fn new_text_node(text: impl AsRef<str>) -> Result<Stanza> {
        let mut text_node = Stanza::new();
        text_node.set_text(text)?;
        Ok(text_node)
    }
}
