use crate::client::muc::room_config::RoomConfig;
use anyhow::Result;
use jid::BareJid;
use minidom::Element;
use prose_xmpp::mods::muc::RoomConfigResponse;
use prose_xmpp::mods::{muc, Caps, MUC};
use prose_xmpp::Client as XMPPClient;

#[derive(Clone)]
pub struct Service {
    pub jid: BareJid,
    pub user_jid: BareJid,
    pub(in crate::client) client: XMPPClient,
}

impl Service {
    pub async fn load_public_rooms(&self) -> Result<Vec<muc::Room>> {
        let muc = self.client.get_mod::<MUC>();
        muc.load_public_rooms(&self.jid).await
    }

    pub async fn create_public_channel(
        &self,
        channel_name: impl AsRef<str>,
    ) -> Result<(), muc::Error> {
        self.query_room_info(channel_name.as_ref()).await.unwrap();

        match self
            .create_room_with_config(
                channel_name.as_ref(),
                RoomConfig::public_channel(channel_name.as_ref()),
            )
            .await
        {
            Ok(_) => (),
            Err(muc::Error::RoomAlreadyExists) => (),
            Err(error) => return Err(error),
        };
        Ok(())
    }

    // pub async fn create_group(&self) -> Result<()> {
    //     let muc = self.client.get_mod::<MUC>();
    //
    //     muc.create_reserved_room(&self.jid, "new_room", |form| async move {
    //         Ok(RoomConfigResponse::Submit(
    //             RoomConfig::group().populate_form(&form)?,
    //         ))
    //     })
    //     .await
    // }
}

impl Service {
    async fn query_room_info(&self, room_name: impl AsRef<str>) -> Result<()> {
        let caps = self.client.get_mod::<Caps>();
        let room_jid = MUC::build_room_jid_bare(&self.jid, room_name)?;
        let result = caps.query_disco_info(room_jid.clone(), None).await?;
        println!("{}", String::from(&Element::from(result)));

        let result = caps.query_disco_items(room_jid, None).await?;
        println!("{}", String::from(&Element::from(result)));

        Ok(())
    }

    async fn create_room_with_config(
        &self,
        room_name: impl AsRef<str>,
        config: RoomConfig,
    ) -> Result<(), muc::Error> {
        let muc = self.client.get_mod::<MUC>();
        let room_name = room_name.as_ref().to_string();
        let nickname = self.user_jid.node_str().unwrap_or("unknown");

        muc.create_reserved_room(&self.jid, room_name.clone(), nickname, |form| async move {
            Ok(RoomConfigResponse::Submit(
                RoomConfig::public_channel(room_name).populate_form(&form)?,
            ))
        })
        .await
    }
}
