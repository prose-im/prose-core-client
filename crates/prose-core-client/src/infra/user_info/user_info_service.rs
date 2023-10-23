// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use jid::BareJid;
use xmpp_parsers::hashes::Sha1HexAttribute;

use prose_xmpp::mods;
use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::avatar;

use crate::domain::user_info::models::AvatarMetadata;
use crate::domain::user_info::services::UserInfoService;
use crate::infra::xmpp::XMPPClient;

#[async_trait]
impl UserInfoService for XMPPClient {
    async fn load_latest_avatar_metadata(&self, from: &BareJid) -> Result<Option<AvatarMetadata>> {
        let profile = self.client.get_mod::<mods::Profile>();
        let metadata = profile
            .load_latest_avatar_metadata(from.clone())
            .await?
            .map(Into::into);
        Ok(metadata)
    }

    async fn load_avatar_image(
        &self,
        from: &BareJid,
        image_id: &avatar::ImageId,
    ) -> Result<Option<AvatarData>> {
        let profile = self.client.get_mod::<mods::Profile>();
        let image = profile
            .load_avatar_image(
                from.clone(),
                &Sha1HexAttribute::from_str(&image_id.as_ref())?,
            )
            .await?;
        Ok(image)
    }
}

impl From<avatar::Info> for AvatarMetadata {
    fn from(value: avatar::Info) -> Self {
        AvatarMetadata {
            bytes: value.bytes as usize,
            mime_type: value.r#type,
            checksum: value.id,
            width: value.width.map(u32::from),
            height: value.height.map(u32::from),
            url: value.url,
        }
    }
}
