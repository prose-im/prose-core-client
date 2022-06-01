// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use std::collections::HashMap;

use libstrophe::{Error, Stanza};

use super::namespaces;
use crate::utils::macros::map;

// -- Structures --

pub struct ProseProtocolStanza;

// -- Implementations --

impl ProseProtocolStanza {
    pub fn named_ns<'a>(
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

        node = Self::attributes(node, attributes)?;

        // Append eventual children and return
        Ok(Self::children(node, children)?)
    }

    pub fn named<'a>(
        name: &str,
        attributes: Option<HashMap<&'a str, &'a str>>,
        children: Option<Vec<Stanza>>,
    ) -> Result<Stanza, Error> {
        Self::named_ns(name, None, attributes, children)
    }

    pub fn text(text: &str) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_text(text);

        Ok(node)
    }

    pub fn attributes<'a>(
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

    pub fn children(mut node: Stanza, children: Option<Vec<Stanza>>) -> Result<Stanza, Error> {
        if let Some(children) = children {
            for child in children {
                node.add_child(child)?;
            }
        }

        Ok(node)
    }

    pub fn result(
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

    pub fn error(
        original_stanza: &Stanza,
        error_type: &str,
        error_code: &str,
        condition: &str,
        text: &str,
    ) -> Result<Stanza, Error> {
        let mut reply_stanza = original_stanza.reply();

        reply_stanza.set_stanza_type("error")?;

        reply_stanza.add_child(ProseProtocolStanza::named_ns(
            "error",
            Some(namespaces::NS_CLIENT),
            Some(map! { "code" => error_code, "type" => error_type }),
            Some(vec![
                ProseProtocolStanza::named_ns(condition, Some(namespaces::NS_STANZAS), None, None)?,
                ProseProtocolStanza::named_ns(
                    "text",
                    Some(namespaces::NS_STANZAS),
                    None,
                    Some(vec![ProseProtocolStanza::text(text)?]),
                )?,
            ]),
        )?)?;

        Ok(reply_stanza)
    }
}
