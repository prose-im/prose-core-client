// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use mockall::predicate;
use xmpp_parsers::mam::Fin;
use xmpp_parsers::rsm::SetResult;

use prose_core_client::domain::messaging::models::MessageLikePayload;
use prose_core_client::domain::rooms::models::RoomInternals;
use prose_core_client::domain::rooms::services::RoomFactory;
use prose_core_client::domain::shared::models::RoomType;
use prose_core_client::dtos::Occupant;
use prose_core_client::test::{mock_data, MessageBuilder, MockRoomFactoryDependencies};
use prose_xmpp::{bare, jid};

#[tokio::test]
async fn test_load_messages_with_ids_resolves_real_jids() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let internals = RoomInternals::group(&bare!("room@conference.prose.org")).with_occupants([
        (
            jid!("room@conference.prose.org/a"),
            Occupant::owner().set_real_jid(&bare!("a@prose.org")),
        ),
        (
            jid!("room@conference.prose.org/c"),
            Occupant::owner().set_real_jid(&bare!("c@prose.org")),
        ),
    ]);

    deps.message_repo
        .expect_get_all()
        .once()
        .return_once(|_, _| {
            Box::pin(async {
                Ok(vec![
                    MessageBuilder::new_with_index(1)
                        .set_from(&jid!("room@conference.prose.org/a"))
                        .build_message_like(),
                    MessageBuilder::new_with_index(2)
                        .set_from(&jid!("room@conference.prose.org/b"))
                        .build_message_like(),
                    MessageBuilder::new_with_index(3)
                        .set_from(&jid!("room@conference.prose.org/c"))
                        .build_message_like(),
                ])
            })
        });

    let room = RoomFactory::from(deps)
        .build(Arc::new(internals))
        .to_generic_room();

    assert_eq!(
        room.load_messages_with_ids(&[
            &MessageBuilder::id_for_index(1),
            &MessageBuilder::id_for_index(2),
            &MessageBuilder::id_for_index(3)
        ])
        .await?,
        vec![
            MessageBuilder::new_with_index(1)
                .set_from(&jid!("a@prose.org"))
                .build_message(),
            MessageBuilder::new_with_index(2)
                .set_from(&jid!("room@conference.prose.org/b"))
                .build_message(),
            MessageBuilder::new_with_index(3)
                .set_from(&jid!("c@prose.org"))
                .build_message(),
        ]
    );

    Ok(())
}

#[tokio::test]
async fn test_load_latest_messages_resolves_real_jids() -> Result<()> {
    let mut deps = MockRoomFactoryDependencies::default();

    let internals = RoomInternals::group(&bare!("room@conference.prose.org")).with_occupants([
        (
            jid!("room@conference.prose.org/a"),
            Occupant::owner().set_real_jid(&bare!("a@prose.org")),
        ),
        (
            jid!("room@conference.prose.org/c"),
            Occupant::owner().set_real_jid(&bare!("c@prose.org")),
        ),
    ]);

    deps.message_archive_service
        .expect_load_messages()
        .once()
        .return_once(|_, _, _, _| {
            Box::pin(async {
                Ok((
                    vec![
                        MessageBuilder::new_with_index(1)
                            .set_from(&jid!("room@conference.prose.org/a"))
                            .build_archived_message("q1"),
                        MessageBuilder::new_with_index(2)
                            .set_from(&jid!("room@conference.prose.org/b"))
                            .build_archived_message("q1"),
                        MessageBuilder::new_with_index(3)
                            .set_from(&jid!("room@conference.prose.org/c"))
                            .build_archived_message("q1"),
                    ],
                    Fin {
                        complete: Default::default(),
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

    let room = RoomFactory::from(deps)
        .build(Arc::new(internals))
        .to_generic_room();

    assert_eq!(
        room.load_latest_messages().await?,
        vec![
            MessageBuilder::new_with_index(1)
                .set_from(&jid!("a@prose.org"))
                .build_message(),
            MessageBuilder::new_with_index(2)
                .set_from(&jid!("room@conference.prose.org/b"))
                .build_message(),
            MessageBuilder::new_with_index(3)
                .set_from(&jid!("c@prose.org"))
                .build_message(),
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
                    .set_from(&mock_data::account_jid().into())
                    .build_message_like_with_payload(
                        1,
                        MessageLikePayload::Reaction {
                            emojis: vec!["🍻".into()],
                        },
                    ),
                MessageBuilder::new_with_index(3)
                    .set_from(&mock_data::account_jid().into())
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

    let internals = RoomInternals::group(&bare!("room@conference.prose.org"));

    let room = RoomFactory::from(deps)
        .build(Arc::new(internals))
        .to_generic_room();

    room.toggle_reaction_to_message(MessageBuilder::id_for_index(1), "🍕".into())
        .await?;

    Ok(())
}
