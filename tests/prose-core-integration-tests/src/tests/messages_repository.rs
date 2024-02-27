// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use pretty_assertions::assert_eq;

use prose_core_client::domain::messaging::models::MessageLikePayload;
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::shared::models::{RoomId, UserId};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::MessageBuilder;
use prose_core_client::user_id;

use crate::tests::{async_test, store};

#[async_test]
async fn test_can_insert_same_message_twice() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));
    let message = MessageBuilder::new_with_index(123).build_message_like();

    repo.append(&room_id, &[message.clone()]).await?;
    repo.append(&room_id, &[message.clone()]).await?;

    assert_eq!(
        repo.get_all(&room_id, &[message.id.clone().into_original_id().unwrap()])
            .await?,
        vec![message]
    );

    Ok(())
}

#[async_test]
async fn test_loads_message_with_reactions() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(3)
        .set_from(user_id!("b@prose.org"))
        .build_reaction_to(1, &["🍿".into(), "📼".into()]);

    let messages = vec![message1, message2];

    repo.append(&room_id, messages.as_slice()).await?;

    let mut message = MessageBuilder::new_with_index(1).build_message();
    message.toggle_reaction(&user_id!("b@prose.org").into(), "🍿".into());
    message.toggle_reaction(&user_id!("b@prose.org").into(), "📼".into());

    assert_eq!(
        repo.get_all(&room_id, &[MessageBuilder::id_for_index(1)])
            .await?,
        messages
    );

    Ok(())
}

#[async_test]
async fn test_load_messages_targeting() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(2).build_message_like();
    let message3 = MessageBuilder::new_with_index(3)
        .set_target_message_idx(1)
        .set_payload(MessageLikePayload::Retraction {})
        .build_message_like();
    let message4 = MessageBuilder::new_with_index(4)
        .set_from(user_id!("b@prose.org"))
        .build_reaction_to(2, &["🍿".into(), "📼".into()]);
    let message5 = MessageBuilder::new_with_index(5).build_message_like();
    let message6 = MessageBuilder::new_with_index(6)
        .set_from(user_id!("c@prose.org"))
        .build_reaction_to(2, &["🍕".into()]);
    let message7 = MessageBuilder::new_with_index(7)
        .set_target_message_idx(5)
        .set_payload(MessageLikePayload::ReadReceipt)
        .build_message_like();

    let messages = vec![
        message1, message2, message3, message4, message5, message6, message7,
    ];

    repo.append(&room_id, messages.as_slice()).await?;

    assert_eq!(
        repo.get_all(
            &room_id,
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2),
                MessageBuilder::id_for_index(5)
            ]
        )
        .await?,
        messages
    );

    Ok(())
}

#[async_test]
async fn test_load_only_messages_targeting() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(2).build_message_like();

    let message3 = MessageBuilder::new_with_index(3)
        .set_target_message_idx(1)
        .set_payload(MessageLikePayload::Retraction {})
        .set_timestamp(Utc.with_ymd_and_hms(2024, 01, 01, 0, 0, 0).unwrap())
        .build_message_like();
    let message4 = MessageBuilder::new_with_index(4)
        .set_from(user_id!("b@prose.org"))
        .set_timestamp(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap())
        .build_reaction_to(1, &["🍿".into(), "📼".into()]);
    let message5 = MessageBuilder::new_with_index(5).build_message_like();
    let message6 = MessageBuilder::new_with_index(6)
        .set_from(user_id!("c@prose.org"))
        .set_timestamp(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap())
        .build_reaction_to(2, &["🍕".into()]);
    let message7 = MessageBuilder::new_with_index(7)
        .set_target_message_idx(5)
        .set_payload(MessageLikePayload::ReadReceipt)
        .build_message_like();

    repo.append(
        &room_id,
        &[
            message1.clone(),
            message2.clone(),
            message3.clone(),
            message4.clone(),
            message5.clone(),
            message6.clone(),
            message7.clone(),
        ],
    )
    .await?;

    assert_eq!(
        vec![message4, message6],
        repo.get_messages_targeting(
            &room_id,
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2),
            ],
            &Utc.with_ymd_and_hms(2024, 02, 22, 0, 0, 0).unwrap()
        )
        .await?
    );

    Ok(())
}

#[async_test]
async fn test_load_only_messages_targeting_sort_order() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1)
        .set_target_message_idx(100)
        .set_payload(MessageLikePayload::Retraction {})
        .set_timestamp(Utc.with_ymd_and_hms(2024, 01, 02, 0, 0, 0).unwrap())
        .build_message_like();
    let message2 = MessageBuilder::new_with_index(2)
        .set_target_message_idx(100)
        .set_payload(MessageLikePayload::Retraction {})
        .set_timestamp(Utc.with_ymd_and_hms(2024, 01, 03, 0, 0, 0).unwrap())
        .build_message_like();
    let message3 = MessageBuilder::new_with_index(3)
        .set_target_message_idx(100)
        .set_payload(MessageLikePayload::Retraction {})
        .set_timestamp(Utc.with_ymd_and_hms(2024, 01, 01, 0, 0, 0).unwrap())
        .build_message_like();

    repo.append(
        &room_id,
        &[message1.clone(), message2.clone(), message3.clone()],
    )
    .await?;

    assert_eq!(
        vec![message3, message1, message2],
        repo.get_messages_targeting(
            &room_id,
            &[MessageBuilder::id_for_index(100),],
            &DateTime::<Utc>::default()
        )
        .await?
    );

    Ok(())
}
