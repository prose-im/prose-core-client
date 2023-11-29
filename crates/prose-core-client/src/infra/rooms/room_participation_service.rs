// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::{BareJid, Jid};
use xmpp_parsers::muc::user::Affiliation;

use crate::domain::rooms::models::RoomError;
use crate::domain::rooms::services::RoomParticipationService;
use crate::dtos::RoomId;
use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, MediatedInvite};

use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomParticipationService for XMPPClient {
    async fn invite_users_to_room(
        &self,
        room_jid: &RoomId,
        participants: &[BareJid],
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        // It seems like the server doesn't send invites to each member if you put them into
        // a single mediated invite. So we'll send one for each participant…
        for participant in participants {
            muc_mod
                .send_mediated_invite(
                    room_jid,
                    MediatedInvite {
                        invites: vec![mediated_invite::Invite {
                            from: None,
                            to: Some(Jid::Bare(participant.clone())),
                            reason: None,
                        }],
                        password: None,
                    },
                )
                .await?;
        }

        Ok(())
    }

    async fn grant_membership(
        &self,
        room_jid: &RoomId,
        participant: &BareJid,
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .update_user_affiliations(room_jid, vec![(participant.clone(), Affiliation::Member)])
            .await?;
        Ok(())
    }
}
