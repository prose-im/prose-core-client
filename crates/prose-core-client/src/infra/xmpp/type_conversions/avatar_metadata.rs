// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::stanza::avatar;

use crate::domain::shared::models::AvatarId;
use crate::domain::user_info::models::AvatarMetadata;

impl From<avatar::Info> for AvatarMetadata {
    fn from(value: avatar::Info) -> Self {
        AvatarMetadata {
            bytes: value.bytes as usize,
            mime_type: value.r#type,
            checksum: AvatarId::from_str_unchecked(value.id.as_ref()),
            width: value.width.map(u32::from),
            height: value.height.map(u32::from),
            url: value.url,
        }
    }
}
