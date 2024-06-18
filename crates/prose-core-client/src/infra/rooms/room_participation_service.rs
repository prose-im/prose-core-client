// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use async_trait::async_trait;
use jid::Jid;
use xmpp_parsers::muc::user::Affiliation;

use prose_xmpp::mods;
use prose_xmpp::stanza::muc::{mediated_invite, MediatedInvite};

use crate::domain::rooms::models::RoomError;
use crate::domain::rooms::services::RoomParticipationService;
use crate::domain::shared::models::MucId;
use crate::dtos::UserId;
use crate::infra::xmpp::XMPPClient;

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[async_trait]
impl RoomParticipationService for XMPPClient {
    async fn invite_users_to_room(
        &self,
        room_id: &MucId,
        participants: &[UserId],
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();

        // It seems like the server doesn't send invites to each member if you put them into
        // a single mediated invite. So we'll send one for each participant…
        for participant in participants {
            muc_mod
                .send_mediated_invite(
                    room_id,
                    MediatedInvite {
                        invites: vec![mediated_invite::Invite {
                            from: None,
                            to: Some(Jid::from(participant.clone().into_inner())),
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
        room_id: &MucId,
        participant: &UserId,
    ) -> Result<(), RoomError> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod
            .update_user_affiliations(
                room_id,
                vec![(participant.clone().into_inner(), Affiliation::Member)],
            )
            .await?;
        Ok(())
    }
}
