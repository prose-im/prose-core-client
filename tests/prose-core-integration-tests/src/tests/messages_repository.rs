// prose-core-client/prose-core-integration-tests
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;

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

    let message = MessageBuilder::new_with_index(1).build_message_like();
    let reaction = MessageBuilder::new_with_index(3)
        .build_reaction_to(message.id.id(), &["🍿".into(), "📼".into()]);

    repo.append(&room_id, &[&message, &reaction]).await?;

    let mut message = MessageBuilder::new_with_index(1).build_message();
    message.toggle_reaction(&reaction.from, "🍿".into());
    message.toggle_reaction(&reaction.from, "📼".into());

    assert_eq!(
        repo.get_all(&room_id, &[message.id.as_ref().unwrap()])
            .await?,
        vec![message]
    );

    Ok(())
}

// #[async_test]
// async fn test_load_messages_targeting() -> Result<()> {
//     let cache = cache().await?;
//
//     let messages = [
//         MessageLike {
//             id: "1000".into(),
//             stanza_id: None,
//             target: None,
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Message {
//                 body: String::from(""),
//             },
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "1001".into(),
//             stanza_id: None,
//             target: None,
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 07, 17, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Message {
//                 body: String::from(""),
//             },
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "1".into(),
//             stanza_id: None,
//             target: Some("1000".into()),
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "2".into(),
//             stanza_id: None,
//             target: Some("1001".into()),
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 17, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "3".into(),
//             stanza_id: None,
//             target: Some("2000".into()),
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 18, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "4".into(),
//             stanza_id: None,
//             target: Some("1000".into()),
//             to: Some(BareJid::from_str("b@prose.org").unwrap()),
//             from: BareJid::from_str("a@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 19, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "5".into(),
//             stanza_id: None,
//             target: Some("1000".into()),
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("c@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 20, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//         MessageLike {
//             id: "6".into(),
//             stanza_id: None,
//             target: Some("1000".into()),
//             to: Some(BareJid::from_str("a@prose.org").unwrap()),
//             from: BareJid::from_str("b@prose.org").unwrap(),
//             timestamp: Utc
//                 .with_ymd_and_hms(2023, 04, 08, 21, 00, 00)
//                 .unwrap()
//                 .into(),
//             payload: Payload::Retraction,
//             is_first_message: false,
//         },
//     ];
//
//     cache.insert_messages(&messages).await?;
//
//     assert_eq!(
//         cache
//             .load_messages_targeting(
//                 &BareJid::from_str("b@prose.org").unwrap(),
//                 &[message::Id::from("1000"), message::Id::from("1001")],
//                 &message::Id::from("1"),
//                 false
//             )
//             .await?,
//         vec![
//             messages[3].clone(),
//             messages[5].clone(),
//             messages[7].clone(),
//         ]
//     );
//
//     assert_eq!(
//         cache
//             .load_messages_targeting(
//                 &BareJid::from_str("b@prose.org").unwrap(),
//                 &[message::Id::from("1000"), message::Id::from("1001")],
//                 None,
//                 true
//             )
//             .await?,
//         vec![
//             messages[0].clone(),
//             messages[1].clone(),
//             messages[2].clone(),
//             messages[3].clone(),
//             messages[5].clone(),
//             messages[7].clone(),
//         ]
//     );
//
//     Ok(())
// }
