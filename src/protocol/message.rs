// prose-core-client
//
// Copyright: 2022, Valerian Saliou <valerian@valeriansaliou.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

// -- Imports --

use libstrophe::{Connection, Stanza};

// -- Structures --

pub struct ProseProtocolMessage;

// -- Implementations --

impl ProseProtocolMessage {
    pub fn handle(connection: &mut Connection, stanza: &Stanza) {
        // TODO
    }
}
