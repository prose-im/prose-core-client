// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Error, Stanza};

// -- Structures --

pub struct ProseProtocolBuilders;

// -- Implementations --

impl ProseProtocolBuilders {
    pub fn stanza_named_ns(name: &str, ns: &str, children: Vec<Stanza>) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_name(name)?;
        node.set_ns(ns)?;

        // Append eventual children and return
        Ok(Self::stanza_children(node, children)?)
    }

    pub fn stanza_named(name: &str, children: Vec<Stanza>) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_name(name)?;

        // Append eventual children and return
        Ok(Self::stanza_children(node, children)?)
    }

    pub fn stanza_text(text: &str) -> Result<Stanza, Error> {
        let mut node = Stanza::new();

        node.set_text(text);

        Ok(node)
    }

    pub fn stanza_children(mut node: Stanza, children: Vec<Stanza>) -> Result<Stanza, Error> {
        for child in children {
            node.add_child(child)?;
        }

        Ok(node)
    }

    pub fn stanza_reply(original_stanza: &Stanza, children: Vec<Stanza>) -> Result<Stanza, Error> {
        let mut reply_stanza = original_stanza.reply();

        reply_stanza.set_stanza_type("result")?;

        // Append eventual children and return
        for child in children {
            reply_stanza.add_child(child)?;
        }

        Ok(reply_stanza)
    }
}
