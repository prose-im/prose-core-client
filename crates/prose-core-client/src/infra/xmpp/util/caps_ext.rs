// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use xmpp_parsers::caps::Caps;

use crate::domain::shared::models::CapabilitiesId;
use crate::domain::user_info::models::JabberClient;

pub trait CapsExt {
    fn client(&self) -> Option<JabberClient>;
    fn id(&self) -> CapabilitiesId;
}

impl CapsExt for Caps {
    fn client(&self) -> Option<JabberClient> {
        self.node.parse().ok()
    }

    fn id(&self) -> CapabilitiesId {
        CapabilitiesId::new(&self.node, self.hash.to_base64())
    }
}
