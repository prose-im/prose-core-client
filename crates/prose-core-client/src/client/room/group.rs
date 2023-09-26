// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use super::Room;
use crate::avatar_cache::AvatarCache;
use crate::data_cache::DataCache;
use anyhow::Result;
use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, MediatedInvite};
use tracing::info;

pub struct Group;

impl<D: DataCache, A: AvatarCache> Room<Group, D, A> {
    pub async fn resend_invites_to_members(&self) -> Result<()> {
        info!("Sending invites to group members…");
        let muc_mod = self.inner.xmpp.get_mod::<mods::MUC>();
        muc_mod
            .send_mediated_invite(
                &self.inner.jid,
                MediatedInvite {
                    invites: self
                        .inner
                        .members
                        .iter()
                        .map(|participant| mediated_invite::Invite {
                            from: None,
                            to: Some(participant.clone().into()),
                            reason: None,
                        })
                        .collect(),
                    password: None,
                },
            )
            .await
    }
}
