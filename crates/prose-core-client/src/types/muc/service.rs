// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::{BareJid, FullJid};
use minidom::Element;
use sha1::{Digest, Sha1};

use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::{mods, Client as XMPPClient};

use crate::types::muc::RoomConfig;

#[derive(Clone)]
pub(crate) struct Service {
    pub jid: BareJid,
    pub user_jid: BareJid,
    pub(crate) client: XMPPClient,
}

pub(crate) enum CreateRoomResult {
    /// The room was created.
    Created(FullJid),
    /// The room did already exist.
    Joined(FullJid),
    /// An error happenend
    Err(mods::muc::Error),
}

impl Service {
    pub async fn load_public_rooms(&self) -> Result<Vec<mods::muc::Room>> {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        muc_mod.load_public_rooms(&self.jid).await
    }

    pub async fn create_room_with_config(
        &self,
        room_name: impl AsRef<str>,
        config: RoomConfig,
    ) -> CreateRoomResult {
        let muc_mod = self.client.get_mod::<mods::MUC>();
        let room_name = room_name.as_ref().to_string();
        let nickname = self.user_jid.node_str().unwrap_or("unknown-user");

        let room_jid = match mods::MUC::build_room_jid_full(&self.jid, &room_name, &nickname) {
            Ok(room_jid) => room_jid,
            Err(err) => return CreateRoomResult::Err(err.into()),
        };

        let result = muc_mod
            .create_reserved_room(&self.jid, room_name.clone(), nickname, |form| async move {
                Ok(RoomConfigResponse::Submit(config.populate_form(&form)?))
            })
            .await;

        match result {
            Ok(()) => CreateRoomResult::Created(room_jid),
            Err(mods::muc::Error::RoomAlreadyExists) => CreateRoomResult::Joined(room_jid),
            Err(err) => CreateRoomResult::Err(err),
        }
    }

    pub async fn query_room_info(&self, room_name: impl AsRef<str>) -> Result<()> {
        let caps = self.client.get_mod::<mods::Caps>();
        let room_jid = mods::MUC::build_room_jid_bare(&self.jid, room_name)?;
        let result = caps.query_disco_info(room_jid.clone(), None).await?;
        println!("{}", String::from(&Element::from(result)));

        let result = caps.query_disco_items(room_jid, None).await?;
        println!("{}", String::from(&Element::from(result)));

        Ok(())
    }
}

impl Service {
    pub fn group_name_for_participants<'a>(
        participants: impl IntoIterator<Item = &'a BareJid>,
    ) -> String {
        let mut sorted_participant_jids = participants
            .into_iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        sorted_participant_jids.sort();

        let mut hasher = Sha1::new();
        hasher.update(sorted_participant_jids.join(","));
        format!("org.prose.group.{:x}", hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prose_xmpp::jid_str;

    #[test]
    fn test_group_name_for_participants() {
        assert_eq!(
            Service::group_name_for_participants(&[
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]),
            "org.prose.group.7c138d7281db96e0d42fe026a4195c85a7dc2cae".to_string()
        );

        assert_eq!(
            Service::group_name_for_participants(&[
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare(),
                jid_str!("c@prose.org").into_bare()
            ]),
            Service::group_name_for_participants(&[
                jid_str!("c@prose.org").into_bare(),
                jid_str!("a@prose.org").into_bare(),
                jid_str!("b@prose.org").into_bare()
            ])
        )
    }
}
