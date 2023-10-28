// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

use prose_core_client::domain::messaging::models::MessageLikePayload;
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::MessageBuilder;
use prose_xmpp::bare;

use crate::tests::{async_test, store};

#[async_test]
async fn test_can_insert_same_message_twice() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = bare!("a@prose.org");
    let message = MessageBuilder::new_with_index(123).build_message_like();

    repo.append(&room_id, &[&message]).await?;
    repo.append(&room_id, &[&message]).await?;

    let message = MessageBuilder::new_with_index(123).build_message();
    assert_eq!(
        repo.get_all(&room_id, &[message.id.as_ref().unwrap()])
            .await?,
        vec![message]
    );

    Ok(())
}

#[async_test]
async fn test_loads_message_with_reactions() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = bare!("a@prose.org");

    repo.append(
        &room_id,
        &[
            &MessageBuilder::new_with_index(1).build_message_like(),
            &MessageBuilder::new_with_index(3)
                .set_from(&bare!("b@prose.org"))
                .build_reaction_to(1, &["🍿".into(), "📼".into()]),
        ],
    )
    .await?;

    let mut message = MessageBuilder::new_with_index(1).build_message();
    message.toggle_reaction(&bare!("b@prose.org"), "🍿".into());
    message.toggle_reaction(&bare!("b@prose.org"), "📼".into());

    assert_eq!(
        repo.get_all(&room_id, &[&MessageBuilder::id_for_index(1)])
            .await?,
        vec![message]
    );

    Ok(())
}

#[async_test]
async fn test_load_messages_targeting() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = bare!("a@prose.org");

    repo.append(
        &room_id,
        &[
            &MessageBuilder::new_with_index(1).build_message_like(),
            &MessageBuilder::new_with_index(2).build_message_like(),
            &MessageBuilder::new_with_index(3)
                .build_message_like_with_payload(1, MessageLikePayload::Retraction {}),
            &MessageBuilder::new_with_index(4)
                .set_from(&bare!("b@prose.org"))
                .build_reaction_to(2, &["🍿".into(), "📼".into()]),
            &MessageBuilder::new_with_index(5).build_message_like(),
            &MessageBuilder::new_with_index(6)
                .set_from(&bare!("c@prose.org"))
                .build_reaction_to(2, &["🍕".into()]),
            &MessageBuilder::new_with_index(7)
                .build_message_like_with_payload(5, MessageLikePayload::ReadReceipt),
        ],
    )
    .await?;

    let mut message2 = MessageBuilder::new_with_index(2).build_message();
    message2.toggle_reaction(&bare!("b@prose.org"), "🍿".into());
    message2.toggle_reaction(&bare!("b@prose.org"), "📼".into());
    message2.toggle_reaction(&bare!("c@prose.org"), "🍕".into());

    let mut message5 = MessageBuilder::new_with_index(5).build_message();
    message5.is_read = true;

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
        vec![message2, message5]
    );

    Ok(())
}
