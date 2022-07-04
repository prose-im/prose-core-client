use crate::error::Result;
use libstrophe::Stanza;

pub(crate) trait StanzaExt {
    /// The ns filter matches the namespace ('xmlns' attribute) of either the top level stanza or
    /// any of it's immediate children (this allows you do handle specific <iq/> stanzas based on
    /// the <query/> child namespace.
    fn has_namespace(&self, ns: &str) -> bool;
    fn new_query(ns: &str) -> Result<Stanza>;
}

impl StanzaExt for Stanza {
    fn has_namespace(&self, ns: &str) -> bool {
        if self.ns() == Some(ns) {
            return true;
        }
        for child in self.children() {
            if child.ns() == Some(ns) {
                return true;
            }
        }
        false
    }

    fn new_query(ns: &str) -> Result<Self> {
        let mut query = Stanza::new();
        query.set_name("query")?;
        query.set_ns(ns)?;
        return Ok(query);
    }
}
