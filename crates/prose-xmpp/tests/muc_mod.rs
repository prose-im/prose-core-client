// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use jid::BareJid;
use minidom::Element;
use pretty_assertions::assert_eq;
use std::str::FromStr;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Item, Role, Status};
use xmpp_parsers::occupant_id::OccupantId;
use xmpp_parsers::presence::Presence;

use prose_proc_macros::mt_test;
use prose_xmpp::mods::muc::RoomOccupancy;
use prose_xmpp::stanza::muc::MucUser;
use prose_xmpp::stanza::Message;
use prose_xmpp::test::{BareJidTestAdditions, ClientTestAdditions, ConnectedClient};
use prose_xmpp::{full, jid, mods, ns, Client};

#[mt_test]
async fn test_collects_presences_and_message_history() -> Result<()> {
    let ConnectedClient {
        connection,
        client,
        sent_events,
        ..
    } = Client::connected_client().await?;

    let presence1 = Presence::new(Default::default())
        .with_from(full!("room@conf.prose.org/user_a"))
        .with_to(BareJid::ours())
        .with_payload(OccupantId {
            id: "occ_1".to_string(),
        })
        .with_payload(MucUser::new().with_item(Item::new(Affiliation::Member, Role::Moderator)));

    let presence2 = Presence::new(Default::default())
        .with_from(full!("room@conf.prose.org/user_b"))
        .with_to(BareJid::ours())
        .with_payload(OccupantId {
            id: "occ_2".to_string(),
        })
        .with_payload(MucUser::new().with_item(Item::new(Affiliation::Member, Role::Participant)));

    let self_presence = Presence::new(Default::default())
        .with_from(full!("room@conf.prose.org/me"))
        .with_to(BareJid::ours())
        .with_payload(OccupantId {
            id: "occ_3".to_string(),
        })
        .with_payload(
            MucUser::new()
                .with_item(Item::new(Affiliation::Member, Role::Visitor))
                .with_status(vec![Status::SelfPresence]),
        );

    let message1 = Message::new()
        .set_type(MessageType::Groupchat)
        .set_from(jid!("room@conf.prose.org/user_a"))
        .set_body("Message 1");

    let message2 = Message::new()
        .set_type(MessageType::Groupchat)
        .set_from(jid!("room@conf.prose.org/user_a"))
        .set_body("Message 2");

    let subject = Message::new()
        .set_type(MessageType::Groupchat)
        .set_from(jid!("room@conf.prose.org/admin"))
        .set_subject("Room Subject");

    {
        let elements = vec![
            presence1.clone().into(),
            presence2.clone().into(),
            self_presence.clone().into(),
            message1.clone().into(),
            message2.clone().into(),
            subject.clone().into(),
        ];

        connection.set_stanza_handler(move |elem| {
            assert!(elem.is("presence", ns::JABBER_CLIENT));
            assert_eq!(Some("room@conf.prose.org/me"), elem.attr("to"));
            elements.clone()
        });
    }

    let muc = client.get_mod::<mods::MUC>();
    let occupancy = muc
        .enter_room(&full!("room@conf.prose.org/me"), None, None, None)
        .await?;

    assert_eq!(
        RoomOccupancy {
            user: MucUser::new()
                .with_item(Item::new(Affiliation::Member, Role::Visitor))
                .with_status(vec![Status::SelfPresence]),
            self_presence: Presence::new(Default::default())
                .with_from(full!("room@conf.prose.org/me"))
                .with_to(BareJid::ours())
                .with_payload(OccupantId {
                    id: "occ_3".to_string(),
                }),
            presences: vec![presence1, presence2],
            subject: Some("Room Subject".to_string()),
            message_history: vec![message1, message2],
        },
        occupancy,
    );

    //assert!(sent_events.read().is_empty());

    Ok(())
}

#[mt_test]
async fn test_handles_empty_subject() -> Result<()> {
    let ConnectedClient {
        connection, client, ..
    } = Client::connected_client().await?;

    let self_presence = Presence::new(Default::default())
        .with_from(full!("room@conf.prose.org/me"))
        .with_to(BareJid::ours())
        .with_payload(OccupantId {
            id: "occ_3".to_string(),
        })
        .with_payload(
            MucUser::new()
                .with_item(Item::new(Affiliation::Member, Role::Visitor))
                .with_status(vec![Status::SelfPresence]),
        );

    let subject = Element::from_str(&format!(
        r#"<message 
            xmlns="jabber:client" 
            from="room@conf.prose.org/admin" 
            to="{}" 
            type="groupchat" 
            xml:lang="en"
        >
            <subject />
        </message>
    "#,
        BareJid::ours()
    ))?;

    {
        let elements = vec![self_presence.clone().into(), subject.clone().into()];

        connection.set_stanza_handler(move |elem| {
            assert!(elem.is("presence", ns::JABBER_CLIENT));
            assert_eq!(Some("room@conf.prose.org/me"), elem.attr("to"));
            elements.clone()
        });
    }

    let muc = client.get_mod::<mods::MUC>();
    let occupancy = muc
        .enter_room(&full!("room@conf.prose.org/me"), None, None, None)
        .await?;

    assert_eq!(
        RoomOccupancy {
            user: MucUser::new()
                .with_item(Item::new(Affiliation::Member, Role::Visitor))
                .with_status(vec![Status::SelfPresence]),
            self_presence: Presence::new(Default::default())
                .with_from(full!("room@conf.prose.org/me"))
                .with_to(BareJid::ours())
                .with_payload(OccupantId {
                    id: "occ_3".to_string(),
                }),
            presences: vec![],
            subject: None,
            message_history: vec![],
        },
        occupancy,
    );

    //assert!(sent_events.read().is_empty());

    Ok(())
}
