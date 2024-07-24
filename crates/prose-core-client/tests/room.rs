// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::iter;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use mockall::predicate;
use pretty_assertions::assert_eq;

use prose_core_client::domain::messaging::models::{
    MessageLikeBody, MessageLikePayload, MessageTargetId, Reaction,
};
use prose_core_client::domain::messaging::services::MessagePage;
use prose_core_client::domain::rooms::models::{RegisteredMember, Room, RoomAffiliation};
use prose_core_client::domain::rooms::services::RoomFactory;
use prose_core_client::domain::shared::models::{CachePolicy, MucId, OccupantId, RoomId, UserId};
use prose_core_client::domain::user_info::models::{UserInfo, UserName};
use prose_core_client::dtos::{Availability, MessageResultSet, MessageServerId, Participant};
use prose_core_client::test::{mock_data, MessageBuilder, MockRoomFactoryDependencies};
use prose_core_client::{muc_id, occupant_id, user_id};
use prose_xmpp::jid;
use prose_xmpp::stanza::message::MucUser;

#[tokio::test]
async fn test_load_messages_with_ids_resolves_real_jids() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let internals = Room::group(muc_id!("room@conference.prose.org"))
        .with_members([RegisteredMember {
            user_id: user_id!("a@prose.org"),
            name: Some("Aron Doe".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        }])
        .by_adding_participants([(
            occupant_id!("room@conference.prose.org/b"),
            Participant::owner().set_vcard_name("Bernhard Doe"),
        )]);

    deps.user_info_domain_service
        .expect_get_user_info()
        .once()
        .with(
            predicate::eq(user_id!("c@prose.org")),
            predicate::eq(CachePolicy::ReturnCacheDataDontLoad),
        )
        .return_once(|_, _| {
            Box::pin(async {
                Ok(Some(UserInfo {
                    name: UserName {
                        nickname: Some("Carl Doe".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }))
            })
        });

    deps.message_repo
        .expect_get_all()
        .once()
        .return_once(|_, _, _| {
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
            MessageBuilder::id_for_index(1),
            MessageBuilder::id_for_index(2),
            MessageBuilder::id_for_index(3)
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

    let internals = Room::group(muc_id!("room@conference.prose.org"))
        .with_members([RegisteredMember {
            user_id: user_id!("a@prose.org"),
            name: Some("Aron Doe".to_string()),
            affiliation: RoomAffiliation::Owner,
            is_self: false,
        }])
        .by_adding_participants([(
            occupant_id!("room@conference.prose.org/b"),
            Participant::owner().set_vcard_name("Bernhard Doe"),
        )]);

    deps.user_info_domain_service
        .expect_get_user_info()
        .once()
        .with(
            predicate::eq(user_id!("c@prose.org")),
            predicate::eq(CachePolicy::ReturnCacheDataDontLoad),
        )
        .return_once(|_, _| {
            Box::pin(async {
                Ok(Some(UserInfo {
                    name: UserName {
                        nickname: Some("Carl Doe".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                }))
            })
        });

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(MessagePage {
                    messages: vec![
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
                    is_last: true,
                })
            })
        });

    deps.message_repo
        .expect_append()
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps).build(internals).to_generic_room();

    assert_eq!(
        room.load_latest_messages().await?,
        MessageResultSet {
            messages: vec![
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
            ],
            last_message_id: None
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_toggle_reaction_in_direct_message() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.message_repo
        .expect_get()
        .with(
            predicate::always(),
            predicate::eq(RoomId::User(user_id!("user@prose.org"))),
            predicate::eq(MessageBuilder::id_for_index(1)),
        )
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(vec![
                    MessageBuilder::new_with_index(1).build_message_like(),
                    MessageBuilder::new_with_index(2)
                        .set_from(mock_data::account_jid().into_user_id())
                        .set_target_message_idx(1)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into()],
                        })
                        .build_message_like(),
                    MessageBuilder::new_with_index(3)
                        .set_from(mock_data::account_jid().into_user_id())
                        .set_target_message_idx(1)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into(), "🍕".into(), "✅".into()],
                        })
                        .build_message_like(),
                ])
            })
        });

    deps.messaging_service
        .expect_react_to_chat_message()
        .once()
        .with(
            predicate::eq(user_id!("user@prose.org")),
            predicate::eq(MessageBuilder::id_for_index(1)),
            predicate::eq(vec!["🍻".into(), "✅".into()]),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::direct_message(
            user_id!("user@prose.org"),
            Availability::Available,
        ))
        .to_generic_room();
    room.toggle_reaction_to_message(MessageBuilder::id_for_index(1), "🍕".into())
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_toggle_reaction_in_muc_room() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let message1 = MessageBuilder::new_with_index(1).build_message_like();

    let mut message2 = MessageBuilder::new_with_index(2)
        .set_from(mock_data::account_jid().into_user_id())
        .set_target_message_idx(1)
        .set_payload(MessageLikePayload::Reaction {
            emojis: vec!["🍻".into()],
        })
        .build_message_like();
    message2.target = Some(MessageTargetId::ServerId(
        MessageBuilder::stanza_id_for_index(1),
    ));

    let mut message3 = MessageBuilder::new_with_index(3)
        .set_from(mock_data::account_jid().into_user_id())
        .set_target_message_idx(1)
        .set_payload(MessageLikePayload::Reaction {
            emojis: vec!["🍻".into(), "🍕".into(), "✅".into()],
        })
        .build_message_like();
    message3.target = Some(MessageTargetId::ServerId(
        MessageBuilder::stanza_id_for_index(1),
    ));

    deps.message_repo
        .expect_get()
        .with(
            predicate::always(),
            predicate::eq(RoomId::Muc(muc_id!("room@conference.prose.org"))),
            predicate::eq(MessageBuilder::id_for_index(1)),
        )
        .once()
        .return_once(|_, _, _| Box::pin(async { Ok(vec![message1, message2, message3]) }));

    deps.messaging_service
        .expect_react_to_muc_message()
        .once()
        .with(
            predicate::eq(muc_id!("room@conference.prose.org")),
            predicate::eq(MessageBuilder::stanza_id_for_index(1)),
            predicate::eq(vec!["🍻".into(), "✅".into()]),
        )
        .return_once(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::group(muc_id!("room@conference.prose.org")))
        .to_generic_room();
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
            predicate::eq(muc_id!("room@conference.prose.org")),
            predicate::eq("New Name"),
        )
        .return_once(|_, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(muc_id!("room@conference.prose.org")).with_name("Old Name"))
        .to_generic_room();

    room.set_name("New Name").await?;

    Ok(())
}

#[tokio::test]
async fn test_fills_result_set_when_loading_messages() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.ctx.config.message_page_size = 5;

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, before, page_size| {
            assert_eq!(5, page_size);
            assert!(before.is_none());

            Box::pin(async {
                Ok(MessagePage {
                    messages: vec![
                        MessageBuilder::new_with_index(100)
                            .set_from(user_id!("b@prose.org"))
                            .set_target_message_idx(90)
                            .set_payload(MessageLikePayload::Reaction {
                                emojis: vec!["✅".into()],
                            })
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(101)
                            .set_from(user_id!("a@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 101"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(102)
                            .set_from(user_id!("b@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 102"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(103)
                            .set_from(user_id!("a@prose.org"))
                            .set_target_message_idx(101)
                            .set_payload(MessageLikePayload::Reaction {
                                emojis: vec!["🍕".into()],
                            })
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(104)
                            .set_from(user_id!("a@prose.org"))
                            .set_target_message_idx(102)
                            .set_payload(MessageLikePayload::Reaction {
                                emojis: vec!["🎉".into()],
                            })
                            .build_archived_message("q1", None),
                    ],
                    is_last: false,
                })
            })
        });

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, before, page_size| {
            assert_eq!(5, page_size);
            assert_eq!(Some(&MessageBuilder::stanza_id_for_index(100)), before);

            Box::pin(async {
                Ok(MessagePage {
                    messages: vec![
                        MessageBuilder::new_with_index(90)
                            .set_from(user_id!("a@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 90"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q2", None),
                        MessageBuilder::new_with_index(91)
                            .set_from(user_id!("a@prose.org"))
                            .set_target_message_idx(90)
                            .set_payload(MessageLikePayload::Reaction {
                                emojis: vec!["✅".into()],
                            })
                            .build_archived_message("q2", None),
                        MessageBuilder::new_with_index(92)
                            .set_from(user_id!("b@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 92"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q2", None),
                        MessageBuilder::new_with_index(93)
                            .set_from(user_id!("a@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 93"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q2", None),
                        MessageBuilder::new_with_index(94)
                            .set_from(user_id!("a@prose.org"))
                            .set_payload(MessageLikePayload::Message {
                                body: MessageLikeBody::text("Message 94"),
                                attachments: vec![],
                                encryption_info: None,
                                is_transient: false,
                            })
                            .build_archived_message("q2", None),
                    ],
                    is_last: true,
                })
            })
        });

    deps.user_info_domain_service
        .expect_get_user_info()
        .returning(|_, _| Box::pin(async { Ok(None) }));

    deps.message_repo
        .expect_append()
        .returning(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(muc_id!("room@conference.prose.org")))
        .to_generic_room();

    let result = room.load_latest_messages().await?;

    assert_eq!(None, result.last_message_id.as_ref().map(|id| id.as_ref()));

    assert_eq!(
        vec![
            MessageBuilder::new_with_index(90)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("A")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 90"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .set_reactions([Reaction {
                    emoji: "✅".into(),
                    from: vec![
                        user_id!("a@prose.org").into(),
                        user_id!("b@prose.org").into(),
                    ]
                },])
                .build_message_dto(),
            MessageBuilder::new_with_index(92)
                .set_from(user_id!("b@prose.org"))
                .set_from_name("B")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 92"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .build_message_dto(),
            MessageBuilder::new_with_index(93)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("A")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 93"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .build_message_dto(),
            MessageBuilder::new_with_index(94)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("A")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 94"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .build_message_dto(),
            MessageBuilder::new_with_index(101)
                .set_from(user_id!("a@prose.org"))
                .set_from_name("A")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 101"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .set_reactions([Reaction {
                    emoji: "🍕".into(),
                    from: vec![user_id!("a@prose.org").into()]
                }])
                .build_message_dto(),
            MessageBuilder::new_with_index(102)
                .set_from(user_id!("b@prose.org"))
                .set_from_name("B")
                .set_payload(MessageLikePayload::Message {
                    body: MessageLikeBody::text("Message 102"),
                    attachments: vec![],
                    encryption_info: None,
                    is_transient: false,
                })
                .set_reactions([Reaction {
                    emoji: "🎉".into(),
                    from: vec![user_id!("a@prose.org").into(),]
                }])
                .build_message_dto()
        ],
        result.messages
    );

    Ok(())
}

#[tokio::test]
async fn test_stops_at_max_message_pages_to_load() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.ctx.config.message_page_size = 5;
    deps.ctx.config.max_message_pages_to_load = 2;

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, before, page_size| {
            assert_eq!(5, page_size);
            assert_eq!(None, before);

            Box::pin(async {
                Ok(MessagePage {
                    messages: (96..=99)
                        .into_iter()
                        .map(|idx| {
                            MessageBuilder::new_with_index(idx)
                                .set_target_message_idx(1001)
                                .set_payload(MessageLikePayload::Reaction { emojis: vec![] })
                                .build_archived_message("q1", None)
                        })
                        .chain(iter::once(
                            MessageBuilder::new_with_index(100)
                                .set_from(user_id!("a@prose.org"))
                                .set_payload(MessageLikePayload::Message {
                                    body: MessageLikeBody::text("Message 100"),
                                    attachments: vec![],
                                    encryption_info: None,
                                    is_transient: false,
                                })
                                .build_archived_message("q1", None),
                        ))
                        .collect(),
                    is_last: false,
                })
            })
        });

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, before, page_size| {
            assert_eq!(5, page_size);
            assert_eq!(Some(&MessageBuilder::stanza_id_for_index(96)), before);

            Box::pin(async {
                Ok(MessagePage {
                    messages: (91..=95)
                        .into_iter()
                        .map(|idx| {
                            MessageBuilder::new_with_index(idx)
                                .set_target_message_idx(1001)
                                .set_payload(MessageLikePayload::Reaction { emojis: vec![] })
                                .build_archived_message("q1", None)
                        })
                        .collect(),
                    is_last: false,
                })
            })
        });

    deps.user_info_domain_service
        .expect_get_user_info()
        .returning(|_, _| Box::pin(async { Ok(None) }));

    deps.message_repo
        .expect_append()
        .returning(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(muc_id!("room@conference.prose.org")))
        .to_generic_room();

    let result = room.load_latest_messages().await?;

    assert_eq!(
        Some(MessageBuilder::stanza_id_for_index(91).as_ref()),
        result.last_message_id.as_ref().map(|id| id.as_ref())
    );

    assert_eq!(
        vec![MessageBuilder::new_with_index(100)
            .set_from(user_id!("a@prose.org"))
            .set_from_name("A")
            .set_payload(MessageLikePayload::Message {
                body: MessageLikeBody::text("Message 100"),
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
            })
            .build_message_dto()],
        result.messages
    );

    Ok(())
}

#[tokio::test]
async fn test_stops_at_last_page() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    deps.ctx.config.message_page_size = 100;
    deps.ctx.config.max_message_pages_to_load = 100;

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(MessagePage {
                    messages: (96..=100)
                        .into_iter()
                        .map(|idx| {
                            MessageBuilder::new_with_index(idx).build_archived_message("q1", None)
                        })
                        .collect(),
                    is_last: false,
                })
            })
        });

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, _, _| {
            Box::pin(async {
                Ok(MessagePage {
                    messages: (93..=95)
                        .into_iter()
                        .map(|idx| {
                            MessageBuilder::new_with_index(idx).build_archived_message("q1", None)
                        })
                        .collect(),
                    is_last: true,
                })
            })
        });

    deps.user_info_domain_service
        .expect_get_user_info()
        .returning(|_, _| Box::pin(async { Ok(None) }));

    deps.message_repo
        .expect_append()
        .returning(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(muc_id!("room@conference.prose.org")))
        .to_generic_room();

    let result = room.load_latest_messages().await?;

    assert_eq!(None, result.last_message_id.as_ref().map(|id| id.as_ref()));
    assert_eq!(8, result.messages.len());

    Ok(())
}

#[tokio::test]
async fn test_resolves_targeted_messages_when_loading_messages() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();
    deps.ctx.config.max_message_pages_to_load = 1;

    deps.message_archive_service
        .expect_load_messages_before()
        .once()
        .return_once(|_, before, _| {
            assert!(before.is_some());

            Box::pin(async {
                Ok(MessagePage {
                    messages: vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(2)
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(3)
                            .set_from(user_id!("user@prose.org"))
                            .set_target_message_idx(2)
                            .set_from(user_id!("a@prose.org"))
                            .set_payload(MessageLikePayload::Reaction {
                                emojis: vec!["🍕".into()],
                            })
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(4)
                            .set_from(user_id!("user@prose.org"))
                            .build_archived_message("q1", None),
                        MessageBuilder::new_with_index(5)
                            .set_from(user_id!("user@prose.org"))
                            .set_timestamp(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap())
                            .build_archived_message("q1", None),
                    ],
                    is_last: false,
                })
            })
        });

    deps.message_repo
        .expect_get_messages_targeting()
        .once()
        .with(
            predicate::always(),
            predicate::eq(RoomId::from(muc_id!("room@conference.prose.org"))),
            predicate::eq(vec![
                MessageBuilder::id_for_index(5).into(),
                MessageBuilder::stanza_id_for_index(5).into(),
                MessageBuilder::id_for_index(4).into(),
                MessageBuilder::stanza_id_for_index(4).into(),
                MessageBuilder::id_for_index(2).into(),
                MessageBuilder::stanza_id_for_index(2).into(),
                MessageBuilder::id_for_index(1).into(),
                MessageBuilder::stanza_id_for_index(1).into(),
            ]),
            predicate::eq(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap()),
        )
        .return_once(|_, _, _, _| {
            Box::pin(async {
                Ok(vec![
                    MessageBuilder::new_with_index(6)
                        .set_from(user_id!("b@prose.org"))
                        .set_target_message_idx(1)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["🧩".into()],
                        })
                        .build_message_like(),
                    MessageBuilder::new_with_index(7)
                        .set_from(user_id!("a@prose.org"))
                        .set_target_message_idx(4)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into()],
                        })
                        .build_message_like(),
                    // This should win over message 3 since `get_messages_targeting`
                    // returns newer messages.
                    MessageBuilder::new_with_index(8)
                        .set_from(user_id!("a@prose.org"))
                        .set_target_message_idx(2)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["🍔".into()],
                        })
                        .build_message_like(),
                    MessageBuilder::new_with_index(9)
                        .set_from(user_id!("a@prose.org"))
                        .set_target_message_idx(1)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["❌".into()],
                        })
                        .build_message_like(),
                    // This should win over message 9 since it is newer
                    MessageBuilder::new_with_index(10)
                        .set_from(user_id!("a@prose.org"))
                        .set_target_message_idx(1)
                        .set_payload(MessageLikePayload::Reaction {
                            emojis: vec!["✅".into()],
                        })
                        .build_message_like(),
                ])
            })
        });

    deps.user_info_domain_service
        .expect_get_user_info()
        .returning(|_, _| Box::pin(async { Ok(None) }));

    deps.message_repo
        .expect_append()
        .returning(|_, _, _| Box::pin(async { Ok(()) }));

    let room = RoomFactory::from(deps)
        .build(Room::public_channel(muc_id!("room@conference.prose.org")))
        .to_generic_room();

    let result = room
        .load_messages_before(&MessageServerId::from("some-stanza-id"))
        .await?;

    assert_eq!(
        vec![
            MessageBuilder::new_with_index(1)
                .set_from(user_id!("user@prose.org"))
                .set_from_name("User")
                .set_reactions([
                    Reaction {
                        emoji: "🧩".into(),
                        from: vec![user_id!("b@prose.org").into()]
                    },
                    Reaction {
                        emoji: "✅".into(),
                        from: vec![user_id!("a@prose.org").into()]
                    }
                ])
                .build_message_dto(),
            MessageBuilder::new_with_index(2)
                .set_from(user_id!("user@prose.org"))
                .set_from_name("User")
                .set_reactions([Reaction {
                    emoji: "🍔".into(),
                    from: vec![user_id!("a@prose.org").into()]
                }])
                .build_message_dto(),
            MessageBuilder::new_with_index(4)
                .set_from(user_id!("user@prose.org"))
                .set_from_name("User")
                .set_reactions([Reaction {
                    emoji: "🍻".into(),
                    from: vec![user_id!("a@prose.org").into()]
                }])
                .build_message_dto(),
            MessageBuilder::new_with_index(5)
                .set_from(user_id!("user@prose.org"))
                .set_from_name("User")
                .set_timestamp(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap())
                .build_message_dto()
        ],
        result.messages
    );

    Ok(())
}
