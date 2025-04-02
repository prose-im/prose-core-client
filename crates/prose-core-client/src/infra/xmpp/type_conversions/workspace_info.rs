// prose-core-client/prose-core-client
//
// Copyright: 2025, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::workspace::models::WorkspaceInfo;
use prose_xmpp::stanza::vcard4::PropertyContainer;
use prose_xmpp::stanza::VCard4;
use prose_xmpp::{ParseError, RequestError};

const ACCENT_COLOR_EXTENSION_KEY: &'static str = "x-accent-color";

trait PropertyContainerExt {
    fn accent_color(&self) -> Option<String>;
}

impl PropertyContainerExt for PropertyContainer {
    fn accent_color(&self) -> Option<String> {
        self.get(ACCENT_COLOR_EXTENSION_KEY)
            .first()
            .map(|v| v.text())
    }
}

impl TryFrom<VCard4> for WorkspaceInfo {
    type Error = RequestError;

    fn try_from(mut vcard: VCard4) -> Result<Self, Self::Error> {
        if vcard.fn_.is_empty() {
            return Err(ParseError::Generic {
                msg: "Missing name in Workspace vCard".to_string(),
            }
            .into());
        }

        Ok(WorkspaceInfo {
            name: Some(vcard.fn_.swap_remove(0).value),
            icon: None,
            accent_color: vcard.unknown_properties.accent_color(),
        })
    }
}
