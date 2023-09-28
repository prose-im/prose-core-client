// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use anyhow::Result;
use jid::BareJid;
use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, MediatedInvite};

pub trait Channel {}

pub struct PrivateChannel;
pub struct PublicChannel;

impl Channel for PrivateChannel {}
impl Channel for PublicChannel {}

impl<Kind, D: DataCache, A: AvatarCache> Room<Kind, D, A>
where
    Kind: Channel,
{
    pub async fn invite_users(&self, users: impl IntoIterator<Item = &BareJid>) -> Result<()> {
        let muc = self.inner.xmpp.get_mod::<mods::MUC>();
        muc.send_mediated_invite(
            &self.inner.jid,
            MediatedInvite {
                invites: users
                    .into_iter()
                    .map(|user| mediated_invite::Invite {
                        from: None,
                        to: Some(user.clone().into()),
                        reason: None,
                    })
                    .collect(),
                password: None,
            },
        )
        .await
    }
}
