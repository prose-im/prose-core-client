// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use std::collections::HashMap;

use libstrophe::{Error, Stanza};

// -- Structures --

pub struct ProseProtocolBuilders;

// -- Implementations --

impl ProseProtocolBuilders {
    pub fn stanza_named_ns<'a>(
        name: &str,
        ns: Option<&str>,
        attributes: Option<HashMap<&'a str, &'a str>>,
        children: Option<Vec<Stanza>>,
    ) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_name(name)?;

        if let Some(ns) = ns {
            node.set_ns(ns)?;
        }

        node = Self::stanza_attributes(node, attributes)?;

        // Append eventual children and return
        Ok(Self::stanza_children(node, children)?)
    }

    pub fn stanza_named<'a>(
        name: &str,
        attributes: Option<HashMap<&'a str, &'a str>>,
        children: Option<Vec<Stanza>>,
    ) -> Result<Stanza, Error> {
        Self::stanza_named_ns(name, None, attributes, children)
    }

    pub fn stanza_text(text: &str) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_text(text);

        Ok(node)
    }

    pub fn stanza_attributes<'a>(
        mut node: Stanza,
        attributes: Option<HashMap<&'a str, &'a str>>,
    ) -> Result<Stanza, Error> {
        if let Some(attributes) = attributes {
            for (name, value) in attributes {
                node.set_attribute(name, value);
            }
        }

        Ok(node)
    }

    pub fn stanza_children(
        mut node: Stanza,
        children: Option<Vec<Stanza>>,
    ) -> Result<Stanza, Error> {
        if let Some(children) = children {
            for child in children {
                node.add_child(child)?;
            }
        }

        Ok(node)
    }

    pub fn stanza_reply(
        original_stanza: &Stanza,
        children: Option<Vec<Stanza>>,
    ) -> Result<Stanza, Error> {
        let mut reply_stanza = original_stanza.reply();

        reply_stanza.set_stanza_type("result")?;

        // Append eventual children and return
        if let Some(children) = children {
            for child in children {
                reply_stanza.add_child(child)?;
            }
        }

        Ok(reply_stanza)
    }
}
