// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::{BareJid, FullJid, NodePart, ResourcePart};
use sha1::{Digest, Sha1};
use std::fmt::{Debug, Formatter};
use std::iter;
use std::sync::Arc;
use xmpp_parsers::muc::user::Status;
use xmpp_parsers::stanza_error::DefinedCondition;

use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::{mods, Client as XMPPClient, IDProvider};

use crate::types::muc::{RoomConfig, RoomMetadata, RoomSettings, RoomValidationError};

#[derive(Clone)]
pub(crate) struct Service {
    pub jid: BareJid,
    pub user_jid: BareJid,
    pub client: XMPPClient,
    pub id_provider: Arc<dyn IDProvider>,
}

impl Debug for Service {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Service")
            .field("jid", &self.jid)
            .field("user_jid", &self.user_jid)
            .finish()
    }
}

impl Service {
    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.load_public_rooms(&self.jid).await
    }

    pub async fn create_or_join_group(
        &self,
        group_name: impl AsRef<str>,
        participants: &[BareJid],
    ) -> Result<RoomMetadata> {
        // We'll create a hash of the sorted jids of our participants. This way users will always
        // come back to the exact same group if they accidentally try to create it again. Also
        // other participants (other than the creator of the room) are able to do the same without
        // having a bookmark.
        let group_hash = Self::hash_for_group_with_participants(
            participants.into_iter().chain(iter::once(&self.user_jid)),
        );
        self.create_or_join_room_with_config(
            &group_hash,
            RoomConfig::group(group_name, participants),
            |info| info.features.validate_as_group(),
        )
        .await
    }

    pub async fn create_or_join_private_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomMetadata> {
        // We'll use a random ID for the jid of the private channel. This way different people can
        // create private channels with the same name without creating a conflict. A conflict might
        // also potentially be a security issue if jid would contain sensitive information.
        let channel_id = self.id_provider.new_id();
        self.create_or_join_room_with_config(
            &Self::name_for_private_channel(&channel_id),
            RoomConfig::private_channel(channel_name.as_ref()),
            |info| info.features.validate_as_private_channel(),
        )
        .await
    }

    pub async fn create_or_join_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<RoomMetadata> {
        // Public channels should be able to conflict, i.e. there should only be one channel for
        // any given name. Since these can be discovered publicly and joined by anyone there should
        // be no harm in exposing the name in the jid.
        self.create_or_join_room_with_config(
            &Self::name_for_public_channel(channel_name.as_ref()),
            RoomConfig::public_channel(channel_name.as_ref()),
            |info| info.features.validate_as_public_channel(),
        )
        .await
    }

    async fn create_or_join_room_with_config(
        &self,
        room_name: &str,
        config: RoomConfig,
        validate: impl FnOnce(&RoomSettings) -> Result<(), RoomValidationError>,
    ) -> Result<RoomMetadata> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let nickname = self.user_jid.node_str().unwrap_or("unknown-user");
        let mut attempt = 0;

        // Algo is…
        // 1. Try to create or enter room with given name
        // 2. If server returns "gone" error (room existed once but was deleted in the meantime)
        //    append "#($ATTEMPT)" to room name and continue at 1.
        // 3. Get room info
        // 4. Use 'validate' handler to validate created/joined room with room info
        // 5. If 'validate' returns an error and the room was created by us, delete room and return
        //    error from handler
        // 6. Return final room jid, user and info.

        loop {
            let unique_room_name = if attempt == 0 {
                room_name.to_string()
            } else {
                format!("{}#{}", room_name, attempt)
            };
            attempt += 1;

            let room_jid = Self::build_room_jid_full(&self.jid, unique_room_name, &nickname)?;
            let room_config = config.clone();
            let result = muc_mod
                .create_reserved_room(&room_jid, |form| async move {
                    Ok(RoomConfigResponse::Submit(
                        room_config.populate_form(&form)?,
                    ))
                })
                .await;

            let occupancy = match result {
                Ok(user) => user,
                Err(error) if error.defined_condition() == Some(DefinedCondition::Gone) => continue,
                Err(error) => return Err(error.into()),
            };

            let caps = self.client.get_mod::<mods::Caps>();
            let settings =
                RoomSettings::try_from(caps.query_disco_info(room_jid.to_bare(), None).await?)?;

            match (validate)(&settings) {
                Ok(_) => (),
                Err(error) if occupancy.user.status.contains(&Status::RoomHasBeenCreated) => {
                    _ = muc_mod.destroy_room(&room_jid.to_bare());
                    return Err(error.into());
                }
                Err(error) => return Err(error.into()),
            }

            return Ok(RoomMetadata {
                room_jid,
                occupancy,
                settings: settings,
            });
        }
    }
}

const GROUP_PREFIX: &str = "org.prose.group";
const PRIVATE_CHANNEL_PREFIX: &str = "org.prose.private-channel";
const PUBLIC_CHANNEL_PREFIX: &str = "org.prose.public-channel";

impl Service {
    pub fn hash_for_group_with_participants<'a>(
        participants: impl IntoIterator<Item = &'a BareJid>,
    ) -> String {
        let mut sorted_participant_jids = participants
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        sorted_participant_jids.sort();

        let mut hasher = Sha1::new();
        hasher.update(sorted_participant_jids.join(","));
        format!("{}.{:x}", GROUP_PREFIX, hasher.finalize())
    }

    pub fn name_for_public_channel(channel_name: &str) -> String {
        return format!(
            "{}.{}",
            PUBLIC_CHANNEL_PREFIX,
            channel_name.to_ascii_lowercase().replace(" ", "-")
        );
    }

    pub fn name_for_private_channel(channel_name: &str) -> String {
        return format!(
            "{}.{}",
            PRIVATE_CHANNEL_PREFIX,
            channel_name.to_ascii_lowercase().replace(" ", "-")
        );
    }

    fn build_room_jid_full(
        service: &BareJid,
        room_name: impl AsRef<str>,
        nickname: impl AsRef<str>,
    ) -> Result<FullJid, jid::Error> {
        Ok(FullJid::from_parts(
            Some(&NodePart::new(
                &room_name.as_ref().to_ascii_lowercase().replace(" ", "-"),
            )?),
            &service.domain(),
            &ResourcePart::new(&nickname.as_ref().to_ascii_lowercase().replace(" ", "-"))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prose_xmpp::jid_str;

    #[test]
    fn test_group_name_for_participants() {
        assert_eq!(
            Service::hash_for_group_with_participants(&[
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]),
            "org.prose.group.7c138d7281db96e0d42fe026a4195c85a7dc2cae".to_string()
        );

        assert_eq!(
            Service::hash_for_group_with_participants(&[
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]),
            Service::hash_for_group_with_participants(&[
                jid_str!("c@prose.org").into_bare(),
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare()
            ])
        )
    }
}
