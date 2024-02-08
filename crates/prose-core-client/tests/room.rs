// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use mockall::predicate;
use xmpp_parsers::mam::Fin;
use xmpp_parsers::rsm::SetResult;

use prose_core_client::domain::messaging::models::MessageLikePayload;
use prose_core_client::domain::rooms::models::{RegisteredMember, Room, RoomAffiliation};
use prose_core_client::domain::rooms::services::RoomFactory;
use prose_core_client::domain::shared::models::{OccupantId, RoomId, RoomType, UserId};
use prose_core_client::dtos::Participant;
use prose_core_client::test::{mock_data, MessageBuilder, MockRoomFactoryDependencies};
use prose_core_client::{occupant_id, room_id, user_id};
use prose_xmpp::stanza::message::MucUser;
use prose_xmpp::{bare, jid};

#[tokio::test]
async fn test_load_messages_with_ids_resolves_real_jids() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let internals = Room::group(room_id!("room@conference.prose.org"))
        .with_members([RegisteredMember {
            user_id: user_id!("a@prose.org"),
            name: Some("Aron Doe".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        }])
        .with_participants([(
            occupant_id!("room@conference.prose.org/b"),
            Participant::owner().set_name("Bernhard Doe"),
        )]);

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(user_id!("c@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("Carl Doe".to_string())) }));

    deps.message_repo
        .expect_get_all()
        .once()
        .return_once(|_, _| {
            Box::pin(async {
                Ok(vec![
                    MessageBuilder::new_with_index(1)
                        .set_from(user_id!("a@prose.org"))
                        .build_message_like(),
                    MessageBuilder::new_with_index(2)
                        .set_from(occupant_id!("room@conference.prose.org/b"))
                        .build_message_like(),
                    MessageBuilder::new_with_index(3)
                        .set_from(user_id!("c@prose.org"))
                        .build_message_like(),
                    MessageBuilder::new_with_index(4)
                        .set_from(occupant_id!("room@conference.prose.org/denise_doe"))
                        .build_message_like(),
                ])
            })
        });

    let room = RoomFactory::from(deps).build(internals).to_generic_room();

    assert_eq!(
        room.load_messages_with_ids(&[
            &MessageBuilder::id_for_index(1),
            &MessageBuilder::id_for_index(2),
            &MessageBuilder::id_for_index(3)
        ])
        .await?,
        vec![
            MessageBuilder::new_with_index(1)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("Aron Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(2)
                .set_from(occupant_id!("room@conference.prose.org/b"))
                .set_from_name("Bernhard Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(3)
                .set_from(user_id!("c@prose.org"))
                .set_from_name("Carl Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(4)
                .set_from(occupant_id!("room@conference.prose.org/denise_doe"))
                .set_from_name("Denise Doe")
                .build_message_dto(),
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_load_latest_messages_resolves_real_jids() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let internals = Room::group(room_id!("room@conference.prose.org"))
        .with_members([RegisteredMember {
            user_id: user_id!("a@prose.org"),
            name: Some("Aron Doe".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        }])
        .with_participants([(
            occupant_id!("room@conference.prose.org/b"),
            Participant::owner().set_name("Bernhard Doe"),
        )]);

    deps.user_profile_repo
        .expect_get_display_name()
        .once()
        .with(predicate::eq(user_id!("c@prose.org")))
        .return_once(|_| Box::pin(async { Ok(Some("Carl Doe".to_string())) }));

    deps.message_archive_service
        .expect_load_messages()
        .once()
        .return_once(|_, _, _, _| {
            Box::pin(async {
                Ok((
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(occupant_id!("room@conference.prose.org/a"))
                            .build_archived_message(
                                "q1",
                                Some(MucUser {
                                    jid: Some(jid!("a@prose.org")),
                                    affiliation: Default::default(),
                                    role: Default::default(),
                                }),
                            ),
                        MessageBuilder::new_with_index(2)
                            .set_from(occupant_id!("room@conference.prose.org/b"))
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(3)
                            .set_from(occupant_id!("room@conference.prose.org/c"))
                            .build_archived_message(
                                "q1",
                                Some(MucUser {
                                    jid: Some(jid!("c@prose.org")),
                                    affiliation: Default::default(),
                                    role: Default::default(),
                                }),
                            ),
                        MessageBuilder::new_with_index(4)
                            .set_from(occupant_id!("room@conference.prose.org/denise_doe"))
                            .build_archived_message("q1", None),
                    ],
                    Fin {
                        complete: Default::default(),
                        queryid: None,
                        set: SetResult {
                            first: None,
                            first_index: None,
                            last: None,
                            count: None,
                        },
                    },
                ))
            })
        });

    deps.message_repo
        .expect_append()
        .once()
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps).build(internals).to_generic_room();

    assert_eq!(
        room.load_latest_messages().await?,
        vec![
            MessageBuilder::new_with_index(1)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("Aron Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(2)
                .set_from(occupant_id!("room@conference.prose.org/b"))
                .set_from_name("Bernhard Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(3)
                .set_from(user_id!("c@prose.org"))
                .set_from_name("Carl Doe")
                .build_message_dto(),
            MessageBuilder::new_with_index(4)
                .set_from(occupant_id!("room@conference.prose.org/denise_doe"))
                .set_from_name("Denise Doe")
                .build_message_dto(),
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_toggle_reaction() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.message_repo.expect_get().once().return_once(|_, _| {
        Box::pin(async {
            Ok(vec![
                MessageBuilder::new_with_index(1).build_message_like(),
                MessageBuilder::new_with_index(2)
                    .set_from(mock_data::account_jid().into_user_id())
                    .build_message_like_with_payload(
                        1,
                        MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into()],
                        },
                    ),
                MessageBuilder::new_with_index(3)
                    .set_from(mock_data::account_jid().into_user_id())
                    .build_message_like_with_payload(
                        1,
                        MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into(), "🍕".into(), "✅".into()],
                        },
                    ),
            ])
        })
    });

    deps.messaging_service
        .expect_react_to_message()
        .once()
        .with(
            predicate::eq(bare!("room@conference.prose.org")),
            predicate::eq(RoomType::Group),
            predicate::eq(MessageBuilder::id_for_index(1)),
            predicate::eq(vec!["🍻".into(), "✅".into()]),
        )
        .return_once(|_, _, _, _| Box::pin(async { Ok(()) }));

    let internals = Room::group(room_id!("room@conference.prose.org"));

    let room = RoomFactory::from(deps).build(internals).to_generic_room();

    room.toggle_reaction_to_message(MessageBuilder::id_for_index(1), "🍕".into())
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_renames_channel_in_sidebar() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.sidebar_domain_service
        .expect_rename_item()
        .once()
        .with(
            predicate::eq(room_id!("room@conference.prose.org")),
            predicate::eq("New Name"),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(room_id!("room@conference.prose.org")).with_name("Old Name"))
        .to_generic_room();

    room.set_name("New Name").await?;

    Ok(())
}
