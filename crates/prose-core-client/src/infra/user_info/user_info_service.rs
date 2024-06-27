// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;

use anyhow::Result;
use async_trait::async_trait;
use jid::Jid;
use tracing::warn;
use xmpp_parsers::hashes::Sha1HexAttribute;

use prose_xmpp::mods;
use prose_xmpp::mods::AvatarData;
use prose_xmpp::stanza::avatar;

use crate::domain::shared::models::{AvatarId, UserId};
use crate::domain::user_info::models::AvatarMetadata;
use crate::domain::user_info::services::UserInfoService;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl UserInfoService for XMPPClient {
    async fn load_latest_avatar_metadata(&self, from: &UserId) -> Result<Option<AvatarMetadata>> {
        let profile = self.client.get_mod::<mods::Profile>();

        match profile.load_latest_avatar_metadata(from.as_ref()).await {
            Ok(metadata) => Ok(metadata.map(Into::into)),
            Err(err) if err.is_forbidden_err() => {
                warn!(
                    "You don't have the rights to access the avatar metadata of {}",
                    from
                );
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }

    async fn load_avatar_image(
        &self,
        from: &UserId,
        image_id: &AvatarId,
    ) -> Result<Option<AvatarData>> {
        let profile = self.client.get_mod::<mods::Profile>();

        match profile
            .load_avatar_image(
                Jid::from(from.clone().into_inner()),
                &Sha1HexAttribute::from_str(&image_id.to_string())?,
            )
            .await
        {
            Ok(image) => Ok(image),
            Err(err) if err.is_forbidden_err() => {
                warn!("You don't have the rights to access the avatar of {}", from);
                Ok(None)
            }
            Err(err) => Err(err.into()),
        }
    }
}

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
