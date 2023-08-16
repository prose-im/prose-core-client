// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use jid::BareJid;
use prose_xmpp::stanza::message;
use prose_xmpp::stanza::message::stanza_id;
pub use prose_xmpp::stanza::message::Emoji;
use serde::{Deserialize, Serialize};

use crate::types::message_like::Payload;
use crate::types::MessageLike;

pub type MessageId = message::Id;
pub type StanzaId = stanza_id::Id;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reaction {
    pub emoji: Emoji,
    pub from: Vec<BareJid>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<MessageId>,
    pub stanza_id: Option<StanzaId>,
    pub from: BareJid,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub is_edited: bool,
    pub is_delivered: bool,
    pub reactions: Vec<Reaction>,
}

impl Message {
    pub(crate) fn reducing_messages(
        messages: impl IntoIterator<Item = MessageLike>,
    ) -> Vec<Message> {
        let mut messages_map = IndexMap::new();
        let mut modifiers: Vec<MessageLike> = vec![];

        for msg in messages.into_iter() {
            match msg.payload {
                Payload::Message { body } => {
                    let message_id = msg.id.clone();

                    let message = Message {
                        id: message_id.into_original_id(),
                        stanza_id: msg.stanza_id,
                        from: msg.from,
                        body,
                        timestamp: msg.timestamp.into(),
                        is_read: false,
                        is_edited: false,
                        is_delivered: false,
                        reactions: vec![],
                    };
                    messages_map.insert(msg.id.id().clone(), Some(message));
                }
                _ => modifiers.push(msg),
            }
        }

        for modifier in modifiers.into_iter() {
            let Some(target) = modifier.target else {
                panic!("Missing target in modifier {:?}", modifier);
            };

            let Some(Some(message)) = messages_map.get_mut(&target) else {
                continue;
            };

            match modifier.payload {
                Payload::Correction { body } => {
                    message.body = body;
                    message.is_edited = true
                }
                Payload::DeliveryReceipt => message.is_delivered = true,
                Payload::ReadReceipt => message.is_read = true,
                Payload::Message { .. } => unreachable!(),
                Payload::Reaction { mut emojis } => {
                    // Iterate over all existing reactions
                    'outer: for reaction in &mut message.reactions {
                        let mut idx: i32 = (emojis.len() as i32) - 1;

                        // Iterate over emojis to be applied
                        for emoji in emojis.iter().rev() {
                            // If the emoji is the same as the reaction…
                            if emoji.as_ref() == reaction.emoji.as_ref() {
                                // …add the author if needed
                                if !reaction.from.contains(&modifier.from) {
                                    reaction.from.push(modifier.from.clone())
                                }
                                // Remove the applied emoji from the list of emojis
                                emojis.remove(idx as usize);
                                // Continue with next reaction
                                continue 'outer;
                            }
                            idx -= 1;
                        }

                        // We couldn't find an emoji for this reaction, so remove our author
                        // from it (if needed)…
                        reaction.from.retain(|from| from != &modifier.from);
                    }

                    // Remove all empty reactions
                    message
                        .reactions
                        .retain(|reaction| !reaction.from.is_empty());

                    // For each of the remaining emojis…
                    for emoji in emojis.into_iter() {
                        // …add a new reaction
                        message.reactions.push(Reaction {
                            emoji: emoji.into_inner().into(),
                            from: vec![modifier.from.clone()],
                        })
                    }
                }
                Payload::Retraction => {
                    messages_map.insert(target, None);
                }
            }
        }

        messages_map.into_values().filter_map(|msg| msg).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::{TimeZone, Utc};
    use jid::BareJid;

    use crate::types::MessageLike;

    use super::*;

    #[test]
    fn test_reduces_emojis() {
        let messages = [
            MessageLike {
                id: "1".into(),
                stanza_id: None,
                target: None,
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
                    .unwrap()
                    .into(),
                payload: Payload::Message {
                    body: String::from("Hello World"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "2".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 01)
                    .unwrap()
                    .into(),
                payload: Payload::Reaction {
                    emojis: vec!["👍".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "3".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("c@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 02)
                    .unwrap()
                    .into(),
                payload: Payload::Reaction {
                    emojis: vec!["👍".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "4".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 03)
                    .unwrap()
                    .into(),
                payload: Payload::Reaction {
                    emojis: vec!["👍".into(), "📼".into(), "🍿".into(), "☕️".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "5".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: BareJid::from_str("a@prose.org").unwrap(),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 04)
                    .unwrap()
                    .into(),
                payload: Payload::Reaction {
                    emojis: vec!["📼".into(), "🍿".into()],
                },
                is_first_message: false,
            },
        ];

        let reduced_message = Message::reducing_messages(messages).pop().unwrap();
        assert_eq!(
            reduced_message,
            Message {
                id: Some("1".into()),
                stanza_id: None,
                from: BareJid::from_str("b@prose.org").unwrap(),
                body: "Hello World".to_string(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
                    .unwrap()
                    .into(),
                is_read: false,
                is_edited: false,
                is_delivered: false,
                reactions: vec![
                    Reaction {
                        emoji: "👍".into(),
                        from: vec![BareJid::from_str("c@prose.org").unwrap()]
                    },
                    Reaction {
                        emoji: "📼".into(),
                        from: vec![BareJid::from_str("b@prose.org").unwrap()]
                    },
                    Reaction {
                        emoji: "🍿".into(),
                        from: vec![BareJid::from_str("b@prose.org").unwrap()]
                    }
                ],
            }
        )
    }
}
