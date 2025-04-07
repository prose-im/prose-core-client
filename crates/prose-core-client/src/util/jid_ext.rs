// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use jid::{BareJid, FullJid, Jid};

const PROSE_WORKSPACE_NAME: &str = "prose-workspace";

pub trait ProseWorkspaceJid {
    fn is_prose_workspace(&self) -> bool;
}

impl ProseWorkspaceJid for Jid {
    fn is_prose_workspace(&self) -> bool {
        self.node().map(|n| n.as_str()) == Some(PROSE_WORKSPACE_NAME)
    }
}

impl ProseWorkspaceJid for BareJid {
    fn is_prose_workspace(&self) -> bool {
        self.node().map(|n| n.as_str()) == Some(PROSE_WORKSPACE_NAME)
    }
}

impl ProseWorkspaceJid for FullJid {
    fn is_prose_workspace(&self) -> bool {
        self.node().map(|n| n.as_str()) == Some(PROSE_WORKSPACE_NAME)
    }
}
