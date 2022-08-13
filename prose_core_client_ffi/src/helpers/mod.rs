// prose-core-client
//
// Copyright: 2022, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::error::Result;
use crate::error::{Error, StanzaParseError};
use libstrophe::{Stanza, StanzaRef};

pub(crate) trait StanzaExt {
    /// The ns filter matches the namespace ('xmlns' attribute) of either the top level stanza or
    /// any of it's immediate children (this allows you do handle specific <iq/> stanzas based on
    /// the <query/> child namespace.
    fn has_namespace(&self, ns: impl AsRef<str>) -> bool;

    fn new_query(ns: impl AsRef<str>, query_id: Option<&str>) -> Result<Stanza>;
    fn new_text_node(text: impl AsRef<str>) -> Result<Stanza>;
    fn new_form_field(
        var: impl AsRef<str>,
        value: impl AsRef<str>,
        kind: Option<&str>,
    ) -> Result<Stanza>;

    fn get_required_child_by_name_and_ns(
        &self,
        name: impl AsRef<str>,
        ns: impl AsRef<str>,
    ) -> Result<StanzaRef, Error>;
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

    fn new_query(ns: impl AsRef<str>, query_id: Option<&str>) -> Result<Self> {
        let mut query = Stanza::new();
        query.set_name("query")?;
        query.set_ns(ns)?;
        if let Some(query_id) = query_id {
            query.set_attribute("queryid", query_id)?;
        }
        Ok(query)
    }

    fn new_text_node(text: impl AsRef<str>) -> Result<Stanza> {
        let mut text_node = Stanza::new();
        text_node.set_text(text)?;
        Ok(text_node)
    }

    fn new_form_field(
        var: impl AsRef<str>,
        value: impl AsRef<str>,
        kind: Option<&str>,
    ) -> Result<Stanza> {
        let mut field = Stanza::new();
        field.set_name("field")?;
        field.set_attribute("var", var)?;

        if let Some(kind) = kind {
            field.set_attribute("type", kind)?;
        }

        let mut value_node = Stanza::new();
        value_node.set_name("value")?;
        value_node.add_child(Stanza::new_text_node(value)?)?;
        field.add_child(value_node)?;
        Ok(field)
    }

    fn get_required_child_by_name_and_ns(
        &self,
        name: impl AsRef<str>,
        ns: impl AsRef<str>,
    ) -> Result<StanzaRef, Error> {
        match self.get_child_by_name_and_ns(&name, ns) {
            None => Err(StanzaParseError::missing_child_node(name.as_ref(), self).into()),
            Some(node) => Ok(node),
        }
    }
}
