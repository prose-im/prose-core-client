// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use anyhow::Result;
use chrono::{TimeZone, Utc};
use jid::Jid;
use pretty_assertions::assert_eq;
use xmpp_parsers::chatstates::ChatState;
use xmpp_parsers::date::DateTime as XMPPDateTime;
use xmpp_parsers::delay::Delay;
use xmpp_parsers::mam::QueryId;
use xmpp_parsers::message::MessageType;
use xmpp_parsers::muc::user::{Affiliation, Role};

use prose_core_client::domain::encryption::services::mocks::MockEncryptionDomainService;
use prose_core_client::domain::messaging::models::{
    MessageLike, MessageLikeBody, MessageLikePayload, MessageParser, MessageTargetId, ReplyTo,
};
use prose_core_client::dtos::{
    Attachment, AttachmentType, Mention, OccupantId, ParticipantId, UnicodeScalarIndex, UserId,
};
use prose_core_client::infra::xmpp::util::MessageExt;
use prose_core_client::{occupant_id, user_id};
use prose_proc_macros::mt_test;
use prose_xmpp::mods::chat::Carbon;
use prose_xmpp::stanza::message::mam::ArchivedMessage;
use prose_xmpp::stanza::message::stanza_id::StanzaId;
use prose_xmpp::stanza::message::{Fallback, Forwarded, MucUser, Range, Reply};
use prose_xmpp::stanza::references::Reference;
use prose_xmpp::stanza::Message;
use prose_xmpp::{bare, full, ns};

#[mt_test]
async fn test_parse_chat_message() -> Result<()> {
    let mut reference = Reference::mention(bare!("them@prose.org"));
    reference.begin = Some(6);
    reference.end = Some(11);

    let message = Message::new()
        .set_id("message-id-1".into())
        .set_type(MessageType::Chat)
        .set_to(bare!("me@prose.org"))
        .set_from(full!("them@prose.org/resource"))
        .set_body("Hello @them")
        .add_references([reference]);

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: None,
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello @them".to_string(),
                    html: "<p>Hello @them</p>".to_string().into(),
                    mentions: vec![Mention {
                        user: user_id!("them@prose.org"),
                        range: Some(UnicodeScalarIndex::new(6)..UnicodeScalarIndex::new(11)),
                    }],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_parse_groupchat_message() -> Result<()> {
    let message = Message::new()
        .set_id("message-id-1".into())
        .set_type(MessageType::Groupchat)
        .set_to(full!("me@prose.org/resource"))
        .set_from(full!("room@groups.prose.org/them"))
        .set_body("Hello World")
        .set_stanza_id(StanzaId {
            id: "dekEV-gtF2hrg_iekCjPAlON".into(),
            by: bare!("user@prose.org").into(),
        });

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: Some("dekEV-gtF2hrg_iekCjPAlON".into()),
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::Occupant(occupant_id!("room@groups.prose.org/them")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello World".to_string(),
                    html: "<p>Hello World</p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_parse_sent_carbon_message() -> Result<()> {
    let message = Carbon::Sent(Forwarded {
        delay: None,
        stanza: Some(Box::new(
            Message::new()
                .set_id("message-id-1".into())
                .set_type(MessageType::Chat)
                .set_from(full!("me@prose.org/res1"))
                .set_to(bare!("them@prose.org"))
                .set_body("Hello World")
                .set_chat_state(Some(ChatState::Active))
                .set_stanza_id(StanzaId {
                    id: "Qiuahv1eo3C222uKhOqjPiW0".into(),
                    by: bare!("user@prose.org").into(),
                }),
        )),
    });

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_carbon(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: Some("Qiuahv1eo3C222uKhOqjPiW0".into()),
            target: None,
            to: Some(bare!("them@prose.org")),
            from: ParticipantId::User(user_id!("me@prose.org")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello World".to_string(),
                    html: "<p>Hello World</p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_parse_mam_groupchat_message() -> Result<()> {
    let message = ArchivedMessage {
        id: "FbGQI-iEUNysr8pdD2PP9mmU".into(),
        query_id: Some(QueryId("de4aba65-7b04-40c0-9bd1-e8454f001e37".to_string())),
        forwarded: Forwarded {
            delay: Some(Delay {
                from: None,
                stamp: XMPPDateTime(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap().into()),
                data: None,
            }),
            stanza: Some(Box::new(
                Message::new()
                    .set_id("message-id-1".into())
                    .set_type(MessageType::Groupchat)
                    .set_to(full!("me@prose.org/resource"))
                    .set_from(full!("room@groups.prose.org/them"))
                    .set_body("Hello World"),
            )),
        },
    };

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_mam_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: Some("FbGQI-iEUNysr8pdD2PP9mmU".into()),
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::Occupant(occupant_id!("room@groups.prose.org/them")),
            timestamp: Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello World".to_string(),
                    html: "<p>Hello World</p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_parse_mam_groupchat_message_with_real_jid() -> Result<()> {
    let message = ArchivedMessage {
        id: "FbGQI-iEUNysr8pdD2PP9mmU".into(),
        query_id: Some(QueryId("de4aba65-7b04-40c0-9bd1-e8454f001e37".to_string())),
        forwarded: Forwarded {
            delay: Some(Delay {
                from: None,
                stamp: XMPPDateTime(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap().into()),
                data: None,
            }),
            stanza: Some(Box::new(
                Message::new()
                    .set_id("message-id-1".into())
                    .set_type(MessageType::Groupchat)
                    .set_to(full!("me@prose.org/resource"))
                    .set_from(full!("room@groups.prose.org/them"))
                    .set_body("Hello World")
                    .set_muc_user(MucUser {
                        jid: Some(Jid::from(bare!("them@prose.org"))),
                        affiliation: Affiliation::Member,
                        role: Role::Participant,
                    }),
            )),
        },
    };

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_mam_message(message)
    .await?;

    assert_eq!(
        parsed_message,
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: Some("FbGQI-iEUNysr8pdD2PP9mmU".into()),
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello World".to_string(),
                    html: "<p>Hello World</p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        }
    );

    Ok(())
}

#[mt_test]
async fn test_parse_mam_chat_message() -> Result<()> {
    let message = ArchivedMessage {
        id: "bne6LtG1ev_jIb1oYNA7nxip".into(),
        query_id: Some(QueryId("037927bd-fe98-4dd5-a9e8-aeab2650c343".to_string())),
        forwarded: Forwarded {
            delay: Some(Delay {
                from: None,
                stamp: XMPPDateTime(Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap().into()),
                data: None,
            }),
            stanza: Some(Box::new(
                Message::new()
                    .set_id("message-id-1".into())
                    .set_type(MessageType::Chat)
                    .set_to(bare!("me@prose.org"))
                    .set_from(full!("them@prose.org/resource"))
                    .set_body("Hello World"),
            )),
        },
    };

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_mam_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: Some("bne6LtG1ev_jIb1oYNA7nxip".into()),
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Utc.with_ymd_and_hms(2024, 02, 23, 0, 0, 0).unwrap(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "Hello World".to_string(),
                    html: "<p>Hello World</p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_parse_delayed_message() -> Result<()> {
    let message = Message::new()
        .set_id("message-id-1".into())
        .set_type(MessageType::Chat)
        .set_to(bare!("me@prose.org"))
        .set_from(full!("them@prose.org/resource"))
        .set_delay(Delay {
            from: None,
            stamp: XMPPDateTime(
                Utc.with_ymd_and_hms(2024, 01, 01, 20, 30, 10)
                    .unwrap()
                    .into(),
            ),
            data: None,
        })
        .set_body("Hello");

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: None,
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Utc.with_ymd_and_hms(2024, 01, 01, 20, 30, 10).unwrap(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody::text("Hello"),
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_message_with_attachment_and_empty_body() -> Result<()> {
    let mut message = Message::new()
        .set_id("message-id-1".into())
        .set_type(MessageType::Chat)
        .set_to(bare!("me@prose.org"))
        .set_from(full!("them@prose.org/resource"));
    message.append_attachments(vec![Attachment {
        r#type: AttachmentType::Image { thumbnail: None },
        url: "https://uploads.prose.org/file.jpg".parse()?,
        media_type: mime::IMAGE_JPEG,
        file_name: "file.jpg".to_string(),
        file_size: Some(250),
    }]);

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-1".into()),
            server_id: None,
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody {
                    raw: "".to_string(),
                    html: "<p></p>".to_string().into(),
                    mentions: vec![],
                },
                attachments: vec![Attachment {
                    r#type: AttachmentType::Image { thumbnail: None },
                    url: "https://uploads.prose.org/file.jpg".parse()?,
                    media_type: mime::IMAGE_JPEG,
                    file_name: "file.jpg".to_string(),
                    file_size: Some(250),
                }],
                encryption_info: None,
                is_transient: false,
                reply_to: None,
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_reply() -> Result<()> {
    let message = Message::new()
        .set_id("message-id-2".into())
        .set_type(MessageType::Chat)
        .set_to(bare!("me@prose.org"))
        .set_from(full!("them@prose.org/resource"))
        .set_body("> Hi 🏳️‍🌈!\n> First Line\n> Second Line\nHello there!")
        .set_reply(Reply::new("message-id-1", Some(bare!("them@prose.org"))))
        .set_fallback(Fallback {
            r#for: Some(ns::REPLY.to_string()),
            subjects: vec![],
            bodies: vec![Range {
                start: Some(0),
                end: Some(38),
            }],
        });

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-2".into()),
            server_id: None,
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody::text("Hello there!"),
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: Some(ReplyTo {
                    id: MessageTargetId::RemoteId("message-id-1".into()),
                    to: Some(ParticipantId::User(user_id!("them@prose.org"))),
                    quote: Some("Hi 🏳️‍🌈!\nFirst Line\nSecond Line".to_string())
                }),
            },
        },
        parsed_message
    );

    Ok(())
}

#[mt_test]
async fn test_reply_without_quote() -> Result<()> {
    let message = Message::new()
        .set_id("message-id-2".into())
        .set_type(MessageType::Chat)
        .set_to(bare!("me@prose.org"))
        .set_from(full!("them@prose.org/resource"))
        .set_body("Hello there!")
        .set_reply(Reply::new("message-id-1", Some(bare!("them@prose.org"))));

    let parsed_message = MessageParser::new(
        "local-id-1".into(),
        None,
        Default::default(),
        Arc::new(MockEncryptionDomainService::new()),
        None,
    )
    .parse_message(message)
    .await?;

    assert_eq!(
        MessageLike {
            id: "local-id-1".into(),
            remote_id: Some("message-id-2".into()),
            server_id: None,
            target: None,
            to: Some(bare!("me@prose.org")),
            from: ParticipantId::User(user_id!("them@prose.org")),
            timestamp: Default::default(),
            payload: MessageLikePayload::Message {
                body: MessageLikeBody::text("Hello there!"),
                attachments: vec![],
                encryption_info: None,
                is_transient: false,
                reply_to: Some(ReplyTo {
                    id: MessageTargetId::RemoteId("message-id-1".into()),
                    to: Some(ParticipantId::User(user_id!("them@prose.org"))),
                    quote: None
                }),
            },
        },
        parsed_message
    );

    Ok(())
}
