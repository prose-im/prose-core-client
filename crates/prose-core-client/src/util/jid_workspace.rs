// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::{BareEntityId, ServerId};
use jid::{BareJid, FullJid, Jid, NodePart};
use std::sync::LazyLock;

static PROSE_WORKSPACE_NODE: LazyLock<NodePart> =
    LazyLock::new(|| NodePart::new("prose-workspace").unwrap().into_owned());

pub trait ProseWorkspaceJid {
    fn is_prose_workspace(&self) -> bool;
}

impl ProseWorkspaceJid for Jid {
    fn is_prose_workspace(&self) -> bool {
        self.node() == Some(&*PROSE_WORKSPACE_NODE)
    }
}

impl ProseWorkspaceJid for BareJid {
    fn is_prose_workspace(&self) -> bool {
        self.node() == Some(&*PROSE_WORKSPACE_NODE)
    }
}

impl ProseWorkspaceJid for FullJid {
    fn is_prose_workspace(&self) -> bool {
        self.node() == Some(&*PROSE_WORKSPACE_NODE)
    }
}

impl ServerId {
    pub(crate) fn to_workspace_entity_id(&self) -> BareEntityId {
        BareEntityId::from(BareJid::from_parts(
            Some(&*PROSE_WORKSPACE_NODE),
            self.as_ref().domain(),
        ))
    }
}
