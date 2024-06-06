// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use pretty_assertions::assert_eq;

use prose_core_client::domain::messaging::models::{
    ArchivedMessageRef, MessageLike, MessageLikeId, MessageLikePayload, MessageRef, MessageTargetId,
};
use prose_core_client::domain::messaging::repos::MessagesRepository;
use prose_core_client::domain::shared::models::{AccountId, MucId, RoomId, UserId};
use prose_core_client::infra::messaging::CachingMessageRepository;
use prose_core_client::test::MessageBuilder;
use prose_core_client::{account_id, muc_id, user_id};

use crate::tests::{async_test, store};

#[async_test]
async fn test_can_insert_same_message_twice() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));
    let message = MessageBuilder::new_with_index(123).build_message_like();

    repo.append(
        &account_id!("account@prose.org"),
        &room_id,
        &[message.clone()],
    )
    .await?;
    repo.append(
        &account_id!("account@prose.org"),
        &room_id,
        &[message.clone()],
    )
    .await?;

    assert_eq!(
        vec![message.clone()],
        repo.get_all(
            &account_id!("account@prose.org"),
            &room_id,
            &[message.id.clone().into_original_id().unwrap()]
        )
        .await?,
    );

    Ok(())
}

#[async_test]
async fn test_loads_message_with_reactions() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let message2 = MessageBuilder::new_with_index(2)
        .set_from(user_id!("b@prose.org"))
        .build_reaction_to(1, &["🍿".into(), "📼".into()]);

    let messages = vec![message1, message2];

    repo.append(
        &account_id!("account@prose.org"),
        &room_id,
        messages.as_slice(),
    )
    .await?;

    assert_eq!(
        messages,
        repo.get_all(
            &account_id!("account@prose.org"),
            &room_id,
            &[MessageBuilder::id_for_index(1)]
        )
        .await?
    );
    assert_eq!(
        Vec::<MessageLike>::new(),
        repo.get_all(
            &account_id!("other_account@prose.org"),
            &room_id,
            &[MessageBuilder::id_for_index(1)]
        )
        .await?
    );

    Ok(())
}

#[async_test]
async fn test_loads_groupchat_message_with_reactions() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));

    let message1 = MessageBuilder::new_with_index(1).build_message_like();
    let mut message2 = MessageBuilder::new_with_index(2)
        .set_from(user_id!("b@prose.org"))
        .build_message_like();

    // Reactions in MUC rooms target other messages by their StanzaId.
    message2.target = Some(MessageTargetId::StanzaId(
        MessageBuilder::stanza_id_for_index(1),
    ));
    message2.payload = MessageLikePayload::Reaction {
        emojis: vec!["🍿".into(), "📼".into()],
    };

    let messages = vec![message1, message2];
    repo.append(
        &account_id!("account@prose.org"),
        &room_id,
        messages.as_slice(),
    )
    .await?;

    assert_eq!(
        messages,
        repo.get_all(
            &account_id!("account@prose.org"),
            &room_id,
            &[MessageBuilder::id_for_index(1)]
        )
        .await?
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

    repo.append(
        &account_id!("account@prose.org"),
        &room_id,
        messages.as_slice(),
    )
    .await?;

    assert_eq!(
        messages,
        repo.get_all(
            &account_id!("account@prose.org"),
            &room_id,
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2),
                MessageBuilder::id_for_index(5)
            ]
        )
        .await?
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

    // Throw in a message that targets another message by their StanzaId. This usually happens
    // only in MUC rooms, but our repo should return it as well…
    let mut message6 = MessageBuilder::new_with_index(6)
        .set_from(user_id!("c@prose.org"))
        .set_timestamp(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap())
        .build_message_like();
    message6.target = Some(MessageTargetId::StanzaId(
        MessageBuilder::stanza_id_for_index(2),
    ));
    message6.payload = MessageLikePayload::Reaction {
        emojis: vec!["🍕".into()],
    };

    let message7 = MessageBuilder::new_with_index(7)
        .set_target_message_idx(5)
        .set_payload(MessageLikePayload::ReadReceipt)
        .build_message_like();

    repo.append(
        &account_id!("account@prose.org"),
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
            &account_id!("account@prose.org"),
            &room_id,
            &[
                MessageTargetId::MessageId(MessageBuilder::id_for_index(1)),
                MessageTargetId::StanzaId(MessageBuilder::stanza_id_for_index(1)),
                MessageTargetId::MessageId(MessageBuilder::id_for_index(2)),
                MessageTargetId::StanzaId(MessageBuilder::stanza_id_for_index(2)),
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
        &account_id!("account@prose.org"),
        &room_id,
        &[message1.clone(), message2.clone(), message3.clone()],
    )
    .await?;

    assert_eq!(
        vec![message3, message1, message2],
        repo.get_messages_targeting(
            &account_id!("account@prose.org"),
            &room_id,
            &[MessageTargetId::MessageId(MessageBuilder::id_for_index(
                100
            ))],
            &DateTime::<Utc>::default()
        )
        .await?
    );

    Ok(())
}

#[async_test]
async fn test_resolves_message_id() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("a@prose.org"));
    let message = MessageBuilder::new_with_index(101).build_message_like();

    repo.append(&account_id!("account@prose.org"), &room_id, &[message])
        .await?;

    assert_eq!(
        Some(MessageBuilder::id_for_index(101)),
        repo.resolve_message_id(
            &account_id!("account@prose.org"),
            &room_id,
            &MessageBuilder::stanza_id_for_index(101)
        )
        .await?
    );

    assert_eq!(
        None,
        repo.resolve_message_id(
            &account_id!("account@prose.org"),
            &room_id,
            &MessageBuilder::stanza_id_for_index(1)
        )
        .await?
    );

    Ok(())
}

#[async_test]
async fn test_get_messages_after() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    repo.append(
        &account_id!("a@prose.org"),
        &muc_id!("room2@prose.org").into(),
        &[
            MessageBuilder::new_with_index(1)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 10, 00, 00).unwrap())
                .build_message_like(),
            MessageBuilder::new_with_index(2)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 11, 00, 00).unwrap())
                .build_message_like(),
            MessageBuilder::new_with_index(3)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 12, 00, 00).unwrap())
                .build_message_like(),
        ],
    )
    .await?;

    repo.append(
        &account_id!("a@prose.org"),
        &muc_id!("room1@prose.org").into(),
        &[MessageBuilder::new_with_index(10)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 12, 00, 00).unwrap())
            .build_message_like()],
    )
    .await?;

    repo.append(
        &account_id!("a@prose.org"),
        &muc_id!("room3@prose.org").into(),
        &[MessageBuilder::new_with_index(10)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 12, 00, 00).unwrap())
            .build_message_like()],
    )
    .await?;

    repo.append(
        &account_id!("b@prose.org"),
        &muc_id!("room2@prose.org").into(),
        &[MessageBuilder::new_with_index(100)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 05, 24, 12, 00, 00).unwrap())
            .build_message_like()],
    )
    .await?;

    assert_eq!(
        vec![
            MessageBuilder::id_for_index(1),
            MessageBuilder::id_for_index(2),
            MessageBuilder::id_for_index(3)
        ],
        repo.get_messages_after(
            &account_id!("a@prose.org"),
            &muc_id!("room2@prose.org").into(),
            DateTime::<Utc>::MIN_UTC
        )
        .await?
        .into_iter()
        .map(|m| m.id.into_original_id().unwrap())
        .collect::<Vec<_>>()
    );

    assert_eq!(
        vec![MessageBuilder::id_for_index(3)],
        repo.get_messages_after(
            &account_id!("a@prose.org"),
            &muc_id!("room2@prose.org").into(),
            Utc.with_ymd_and_hms(2024, 05, 24, 11, 00, 00).unwrap()
        )
        .await?
        .into_iter()
        .map(|m| m.id.into_original_id().unwrap())
        .collect::<Vec<_>>()
    );

    Ok(())
}

#[async_test]
async fn test_clears_cache() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    repo.append(
        &account_id!("a@prose.org"),
        &user_id!("room1@prose.org").into(),
        &[MessageBuilder::new_with_index(1).build_message_like()],
    )
    .await?;
    repo.append(
        &account_id!("a@prose.org"),
        &user_id!("room2@prose.org").into(),
        &[MessageBuilder::new_with_index(2).build_message_like()],
    )
    .await?;
    repo.append(
        &account_id!("b@prose.org"),
        &user_id!("room1@prose.org").into(),
        &[MessageBuilder::new_with_index(1).build_message_like()],
    )
    .await?;
    repo.append(
        &account_id!("b@prose.org"),
        &user_id!("room2@prose.org").into(),
        &[MessageBuilder::new_with_index(2).build_message_like()],
    )
    .await?;

    assert_eq!(
        vec![MessageBuilder::new_with_index(1).build_message_like()],
        repo.get_all(
            &account_id!("a@prose.org"),
            &user_id!("room1@prose.org").into(),
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2)
            ]
        )
        .await?
    );
    assert_eq!(
        vec![MessageBuilder::new_with_index(2).build_message_like()],
        repo.get_all(
            &account_id!("a@prose.org"),
            &user_id!("room2@prose.org").into(),
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2)
            ]
        )
        .await?
    );

    repo.clear_cache(&account_id!("a@prose.org")).await?;

    assert!(repo
        .get_all(
            &account_id!("a@prose.org"),
            &user_id!("room1@prose.org").into(),
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2)
            ]
        )
        .await?
        .is_empty());
    assert!(repo
        .get_all(
            &account_id!("a@prose.org"),
            &user_id!("room2@prose.org").into(),
            &[
                MessageBuilder::id_for_index(1),
                MessageBuilder::id_for_index(2),
            ],
        )
        .await?
        .is_empty());

    assert_eq!(
        vec![MessageBuilder::new_with_index(1).build_message_like()],
        repo.get_all(
            &account_id!("b@prose.org"),
            &user_id!("room1@prose.org").into(),
            &[MessageBuilder::id_for_index(1)]
        )
        .await?
    );
    assert_eq!(
        vec![MessageBuilder::new_with_index(2).build_message_like()],
        repo.get_all(
            &account_id!("b@prose.org"),
            &user_id!("room2@prose.org").into(),
            &[MessageBuilder::id_for_index(2)]
        )
        .await?
    );

    Ok(())
}

#[async_test]
async fn test_loads_latest_received_message() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("room@prose.org"));

    repo.append(
        &account_id!("a@prose.org"),
        &room_id,
        &[MessageBuilder::new_with_index(1)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap())
            .build_message_like()],
    )
    .await?;

    repo.append(
        &account_id!("b@prose.org"),
        &room_id,
        &[
            MessageBuilder::new_with_index(2)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap())
                .set_stanza_id(None)
                .build_message_like(),
            MessageBuilder::new_with_index(3)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap())
                .build_message_like(),
            MessageBuilder::new_with_index(4)
                .set_timestamp(Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap())
                .build_message_like(),
        ],
    )
    .await?;

    assert_eq!(
        Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(3),
            timestamp: Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
        }),
        repo.get_last_received_message(&account_id!("b@prose.org"), &room_id, None)
            .await?,
    );

    assert_eq!(
        Some(ArchivedMessageRef {
            stanza_id: MessageBuilder::stanza_id_for_index(4),
            timestamp: Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap(),
        }),
        repo.get_last_received_message(
            &account_id!("b@prose.org"),
            &room_id,
            Some(Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap())
        )
        .await?,
    );

    assert_eq!(
        None,
        repo.get_last_received_message(
            &account_id!("b@prose.org"),
            &RoomId::from(user_id!("void@prose.org")),
            None
        )
        .await?,
    );

    Ok(())
}

#[async_test]
async fn test_loads_latest_message() -> Result<()> {
    let repo = CachingMessageRepository::new(store().await?);

    let room_id = RoomId::from(user_id!("room@prose.org"));

    repo.append(
        &account_id!("a@prose.org"),
        &room_id,
        &[MessageBuilder::new_with_index(1)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 6, 1, 0, 0, 0).unwrap())
            .build_message_like()],
    )
    .await?;

    let mut messages = [
        MessageBuilder::new_with_index(2)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap())
            .build_message_like(),
        MessageBuilder::new_with_index(3)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap())
            .build_message_like(),
        MessageBuilder::new_with_index(4)
            .set_timestamp(Utc.with_ymd_and_hms(2024, 3, 1, 0, 0, 0).unwrap())
            .build_message_like(),
    ];

    messages[0].id = MessageLikeId::new(None);

    repo.append(&account_id!("b@prose.org"), &room_id, &messages)
        .await?;

    assert_eq!(
        Some(MessageRef {
            id: MessageBuilder::id_for_index(3),
            timestamp: Utc.with_ymd_and_hms(2024, 4, 1, 0, 0, 0).unwrap(),
        }),
        repo.get_last_message(&account_id!("b@prose.org"), &room_id)
            .await?,
    );

    assert_eq!(
        None,
        repo.get_last_received_message(
            &account_id!("b@prose.org"),
            &RoomId::from(user_id!("void@prose.org")),
            None
        )
        .await?,
    );

    Ok(())
}
