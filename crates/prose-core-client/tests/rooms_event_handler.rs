// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr;
use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use minidom::Element;
use mockall::predicate;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Item, Role};
use xmpp_parsers::presence::Presence;

use prose_core_client::app::event_handlers::{RoomsEventHandler, XMPPEvent, XMPPEventHandler};
use prose_core_client::domain::rooms::models::{ComposeState, RoomAffiliation, RoomInternals};
use prose_core_client::domain::rooms::services::{CreateOrEnterRoomRequest, RoomFactory};
use prose_core_client::domain::shared::models::RoomId;
use prose_core_client::dtos::{Participant, UserBasicInfo};
use prose_core_client::test::{
    mock_data, ConstantTimeProvider, MockAppDependencies, MockRoomFactoryDependencies,
};
use prose_core_client::{room_id, ClientRoomEventType};
use prose_xmpp::mods::muc;
use prose_xmpp::stanza::muc::{MediatedInvite, MucUser};
use prose_xmpp::{bare, full, jid, mods};

#[tokio::test]
async fn test_handles_presence_for_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::group(room_id!("room@conference.prose.org")));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(bare!("real-jid@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("George Washington".to_string())) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Status(mods::status::Event::Presence(
            Presence::available()
                .with_from(full!("room@conference.prose.org/nick"))
                .with_to(mock_data::account_jid())
                .with_payload(MucUser::new().with_items(vec![Item::new(
                        Affiliation::Member,
                        Role::Participant,
                    )
                    .with_jid(full!("real-jid@prose.org/resource"))])),
        )))
        .await?;

    assert_eq!(room.participants().len(), 1);

    let occupant = room
        .get_participant(&jid!("room@conference.prose.org/nick"))
        .unwrap()
        .clone();

    assert_eq!(
        occupant,
        Participant {
            id: Some(bare!("real-jid@prose.org")),
            name: Some("George Washington".to_string()),
            affiliation: RoomAffiliation::Member,
            compose_state: ComposeState::Idle,
            compose_state_updated: Default::default(),
        }
    );

    Ok(())
}

async fn test_user_entered_room_with_multiple_resources_and_same_nickname() -> Result<()> {
    // <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-GJFeX9yi" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-GJFeX9yi" role="participant" />
    //     <item affiliation="none" jid="cram@prose.org/prose-rJF2R8AI" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram" to="marc@prose.org/prose-QAaMglCf" type="unavailable">
    //     <status>Disconnected: closed</status><occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-rJF2R8AI" role="none" />
    //     </x>
    //     </presence>
    panic!("Implement me")
}

async fn test_user_entered_room_with_multiple_resources_and_different_nicknames() -> Result<()> {
    // <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" xml:lang="en">
    //     <x xmlns='vcard-temp:x:update'>
    //     <photo>cdc05cb9c48d5e817a36d462fe0470a0579e570a</photo>
    //     </x>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="participant" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-CnLZs8Id" to="marc@prose.org/prose-YJlWBFC2" type="unavailable">
    //     <status>Disconnected: closed</status>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-AJJuBKzn" role="none" />
    //     </x>
    //     </presence>
    //
    //     <presence xmlns='jabber:client' from="org.prose.public-channel.dev-core#1@groups.prose.org/cram-t7YjIEMF" to="marc@prose.org/prose-YJlWBFC2" type="unavailable">
    //     <status>Disconnected: closed</status>
    //     <occupant-id xmlns='urn:xmpp:occupant-id:0' id="gk6wmXJJ58Thj95cbfEX1Tzr0ONoOuZyU6SyMAvREXw=" />
    //     <x xmlns='http://jabber.org/protocol/muc#user'>
    // <item affiliation="none" jid="cram@prose.org/prose-IVO5rzuB" role="none" />
    //     </x>
    //     </presence>
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_disconnected_participant() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_added_member() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_kicked_user() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_disconnected_user() -> Result<()> {
    panic!("Implement me")
}

#[tokio::test]
async fn test_handles_destroyed_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.connected_rooms_repo
        .expect_get()
        .once()
        .with(predicate::eq(room_id!("group@prose.org")))
        .return_once(|_| Some(Arc::new(RoomInternals::group(room_id!("group@prose.org")))));

    deps.sidebar_domain_service
        .expect_handle_destroyed_room()
        .once()
        .with(
            predicate::eq(room_id!("group@prose.org")),
            predicate::eq(Some(room_id!("private-channel@prose.org"))),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    let xml = format!(
        r#"<presence xmlns='jabber:client' from="group@prose.org" to="{user}" type="unavailable">
        <x xmlns='http://jabber.org/protocol/muc#user'>
            <destroy jid="private-channel@prose.org" />
            <item affiliation="owner" jid="{user}" role="none" />
            <status code="110" />
        </x>
    </presence>"#,
        user = mock_data::account_jid()
    );

    let presence = Presence::try_from(Element::from_str(&xml)?)?;
    event_handler
        .handle_event(XMPPEvent::Status(mods::status::Event::Presence(presence)))
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_handles_chat_state_for_muc_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(
        RoomInternals::group(room_id!("room@conference.prose.org")).with_participants([(
            jid!("room@conference.prose.org/nickname"),
            Participant::owner()
                .set_real_id(&bare!("nickname@prose.org"))
                .set_name("Janice Doe"),
        )]),
    );

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("room@conference.prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ComposingUsersChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Chat(mods::chat::Event::ChatStateChanged {
            from: jid!("room@conference.prose.org/nickname"),
            chat_state: ChatState::Composing,
            message_type: MessageType::Groupchat,
        }))
        .await?;

    let occupant = room
        .get_participant(&jid!("room@conference.prose.org/nickname"))
        .unwrap()
        .clone();

    assert_eq!(occupant.compose_state, ComposeState::Composing);
    assert_eq!(
        occupant.compose_state_updated,
        Utc.with_ymd_and_hms(2023, 01, 04, 0, 0, 0).unwrap()
    );

    let time_provider = Arc::new(ConstantTimeProvider::ymd_hms(2023, 01, 04, 00, 00, 20));

    let mut factory_deps = MockRoomFactoryDependencies::default();
    factory_deps.time_provider = time_provider.clone();

    let room_factory = RoomFactory::from(factory_deps);
    let room = room_factory.build(room.clone()).to_generic_room();
    assert_eq!(
        room.load_composing_users().await?,
        vec![UserBasicInfo {
            name: "Janice Doe".to_string(),
            id: bare!("nickname@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_chat_state_for_direct_message_room() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    let room = Arc::new(RoomInternals::for_direct_message(
        &mock_data::account_jid().into_bare(),
        &bare!("contact@prose.org"),
        "Janice Doe",
    ));

    {
        let room = room.clone();
        deps.connected_rooms_repo
            .expect_get()
            .once()
            .with(predicate::eq(room_id!("contact@prose.org")))
            .return_once(move |_| Some(room.clone()));
    }
    deps.time_provider = Arc::new(ConstantTimeProvider::ymd(2023, 01, 04));
    deps.client_event_dispatcher
        .expect_dispatch_room_event()
        .once()
        .with(
            predicate::eq(room.clone()),
            predicate::eq(ClientRoomEventType::ComposingUsersChanged),
        )
        .return_once(|_, _| ());

    let event_handler = RoomsEventHandler::from(&deps.into_deps());

    event_handler
        .handle_event(XMPPEvent::Chat(mods::chat::Event::ChatStateChanged {
            from: jid!("contact@prose.org/resource"),
            chat_state: ChatState::Composing,
            message_type: MessageType::Chat,
        }))
        .await?;

    let occupant = room
        .get_participant(&jid!("contact@prose.org"))
        .unwrap()
        .clone();

    assert_eq!(occupant.compose_state, ComposeState::Composing);
    assert_eq!(
        occupant.compose_state_updated,
        Utc.with_ymd_and_hms(2023, 01, 04, 0, 0, 0).unwrap()
    );

    let time_provider = Arc::new(ConstantTimeProvider::ymd_hms(2023, 01, 04, 00, 00, 20));

    let mut factory_deps = MockRoomFactoryDependencies::default();
    factory_deps.time_provider = time_provider.clone();

    let room_factory = RoomFactory::from(factory_deps);
    let room = room_factory.build(room.clone()).to_generic_room();
    assert_eq!(
        room.load_composing_users().await?,
        vec![UserBasicInfo {
            name: "Janice Doe".to_string(),
            id: bare!("contact@prose.org")
        }]
    );

    time_provider.set_ymd_hms(2023, 01, 04, 00, 00, 31);
    assert!(room.load_composing_users().await?.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_handles_invite() -> Result<()> {
    let mut deps = MockAppDependencies::default();

    deps.sidebar_domain_service
        .expect_insert_item_by_creating_or_joining_room()
        .once()
        .with(predicate::eq(CreateOrEnterRoomRequest::JoinRoom {
            room_jid: room_id!("group@conference.prose.org"),
            password: None,
        }))
        .return_once(|_| Box::pin(async move { Ok(room_id!("group@conference.prose.org")) }));

    let event_handler = RoomsEventHandler::from(&deps.into_deps());
    event_handler
        .handle_event(XMPPEvent::MUC(muc::Event::MediatedInvite {
            from: jid!("group@conference.prose.org"),
            invite: MediatedInvite {
                invites: vec![],
                password: None,
            },
        }))
        .await?;

    Ok(())
}
