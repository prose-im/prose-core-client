// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, Jid};

use crate::domain::rooms::models::RoomError;
use crate::domain::rooms::services::RoomParticipationService;
use crate::dtos::RoomJid;
use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, MediatedInvite};

use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomParticipationService for XMPPClient {
    async fn invite_users_to_room(
        &self,
        room_jid: &RoomJid,
        participants: &[&BareJid],
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .send_mediated_invite(
                room_jid,
                MediatedInvite {
                    invites: participants
                        .iter()
                        .map(|participant| mediated_invite::Invite {
                            from: None,
                            to: Some(Jid::Bare((*participant).clone())),
                            reason: None,
                        })
                        .collect(),
                    password: None,
                },
            )
            .await?;
        Ok(())
    }
}
