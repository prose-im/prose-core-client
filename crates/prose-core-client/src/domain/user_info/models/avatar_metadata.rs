// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};

use prose_xmpp::stanza::avatar;

#[derive(Clone, PartialEq, Debug)]
pub struct AvatarMetadata {
    pub bytes: usize,
    pub mime_type: String,
    pub checksum: avatar::ImageId,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AvatarInfo {
    pub checksum: avatar::ImageId,
    pub mime_type: String,
}

impl AvatarMetadata {
    pub fn to_info(&self) -> AvatarInfo {
        AvatarInfo {
            checksum: self.checksum.clone(),
            mime_type: self.mime_type.clone(),
        }
    }

    pub fn into_info(self) -> AvatarInfo {
        AvatarInfo {
            checksum: self.checksum,
            mime_type: self.mime_type,
        }
    }
}
