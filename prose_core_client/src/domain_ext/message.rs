use indexmap::IndexMap;

use prose_core_domain::{Message, Reaction};

use crate::types::message_like::Payload;
use crate::types::MessageLike;

pub trait MessageExt {
    fn reducing_messages(messages: impl IntoIterator<Item = MessageLike>) -> Vec<Message>;
}

impl MessageExt for Message {
    fn reducing_messages(messages: impl IntoIterator<Item = MessageLike>) -> Vec<Message> {
        let mut messages_map = IndexMap::new();
        let mut modifiers: Vec<MessageLike> = vec![];

        for msg in messages.into_iter() {
            match msg.payload {
                Payload::Message { body } => {
                    let message = Message {
                        id: msg.id.as_ref().into(),
                        stanza_id: msg.stanza_id.map(|id| id.as_ref().into()),
                        from: msg.from,
                        body,
                        timestamp: msg.timestamp,
                        is_read: false,
                        is_edited: false,
                        is_delivered: false,
                        reactions: vec![],
                    };
                    messages_map.insert(msg.id, Some(message));
                }
                _ => modifiers.push(msg),
            }
        }

        for modifier in modifiers.into_iter() {
            let Some(target) = modifier.target else {
                panic!("Missing target in modifier {:?}", modifier);
            };

            let Some(Some(message)) = messages_map.get_mut(&target) else {
                continue
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
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 00).unwrap(),
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
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 01).unwrap(),
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
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 02).unwrap(),
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
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 03).unwrap(),
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
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 04).unwrap(),
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
                id: "1".into(),
                stanza_id: None,
                from: BareJid::from_str("b@prose.org").unwrap(),
                body: "Hello World".to_string(),
                timestamp: Utc.with_ymd_and_hms(2023, 04, 07, 16, 00, 00).unwrap(),
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
