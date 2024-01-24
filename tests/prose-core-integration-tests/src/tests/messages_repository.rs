// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::messaging::models::MessageLikePayload;
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::shared::models::{RoomId, UserId};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::MessageBuilder;
use prose_core_client::{room_id, user_id};

use crate::tests::{async_test, store};

#[async_test]
async fn test_can_insert_same_message_twice() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = room_id!("a@prose.org");
    let message = MessageBuilder::new_with_index(123).build_message_like();

    repo.append(&room_id, &[&message]).await?;
    repo.append(&room_id, &[&message]).await?;

    assert_eq!(
        repo.get_all(&room_id, &[&message.id.clone().into_original_id().unwrap()])
            .await?,
        vec![message]
    );

    Ok(())
}

#[async_test]
async fn test_loads_message_with_reactions() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = room_id!("a@prose.org");

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(3)
        .set_from(user_id!("b@prose.org"))
        .build_reaction_to(1, &["🍿".into(), "📼".into()]);

    repo.append(&room_id, &[&message1, &message2]).await?;

    let mut message = MessageBuilder::new_with_index(1).build_message();
    message.toggle_reaction(&user_id!("b@prose.org"), "🍿".into());
    message.toggle_reaction(&user_id!("b@prose.org"), "📼".into());

    assert_eq!(
        repo.get_all(&room_id, &[&MessageBuilder::id_for_index(1)])
            .await?,
        vec![message1, message2]
    );

    Ok(())
}

#[async_test]
async fn test_load_messages_targeting() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = room_id!("a@prose.org");

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(2).build_message_like();
    let message3 = MessageBuilder::new_with_index(3)
        .build_message_like_with_payload(1, MessageLikePayload::Retraction {});
    let message4 = MessageBuilder::new_with_index(4)
        .set_from(user_id!("b@prose.org"))
        .build_reaction_to(2, &["🍿".into(), "📼".into()]);
    let message5 = MessageBuilder::new_with_index(5).build_message_like();
    let message6 = MessageBuilder::new_with_index(6)
        .set_from(user_id!("c@prose.org"))
        .build_reaction_to(2, &["🍕".into()]);
    let message7 = MessageBuilder::new_with_index(7)
        .build_message_like_with_payload(5, MessageLikePayload::ReadReceipt);

    repo.append(
        &room_id,
        &[
            &message1, &message2, &message3, &message4, &message5, &message6, &message7,
        ],
    )
    .await?;

    assert_eq!(
        repo.get_all(
            &room_id,
            &[
                &MessageBuilder::id_for_index(1),
                &MessageBuilder::id_for_index(2),
                &MessageBuilder::id_for_index(5)
            ]
        )
        .await?,
        vec![message1, message2, message3, message4, message5, message6, message7]
    );

    Ok(())
}
