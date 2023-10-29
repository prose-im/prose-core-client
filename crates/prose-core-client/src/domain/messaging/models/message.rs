// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use jid::BareJid;
use serde::{Deserialize, Serialize};

use prose_utils::id_string;
pub use prose_xmpp::stanza::message::Emoji;

use super::{MessageLike, MessageLikePayload};

id_string!(MessageId);
id_string!(StanzaId);

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
    pub fn toggle_reaction(&mut self, user_id: &BareJid, emoji: Emoji) {
        let Some(reaction) = self
            .reactions
            .iter_mut()
            .find(|reaction| reaction.emoji == emoji)
        else {
            self.reactions.push(Reaction {
                emoji,
                from: vec![user_id.clone()],
            });
            return;
        };

        if let Some(idx) = reaction.from.iter().position(|jid| jid == user_id) {
            reaction.from.remove(idx);
        } else {
            reaction.from.push(user_id.clone())
        }
    }

    pub fn reactions_from<'a, 'b: 'a>(
        &'a self,
        user_id: &'b BareJid,
    ) -> impl Iterator<Item = &'a Emoji> {
        self.reactions
            .iter()
            .filter(|reaction| reaction.from.contains(user_id))
            .map(|reaction| &reaction.emoji)
    }
}

impl Message {
    pub(crate) fn reducing_messages(
        messages: impl IntoIterator<Item = MessageLike>,
    ) -> Vec<Message> {
        let mut messages_map = IndexMap::new();
        let mut modifiers: Vec<MessageLike> = vec![];

        for msg in messages.into_iter() {
            match msg.payload {
                MessageLikePayload::Message { body } => {
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
                MessageLikePayload::Correction { body } => {
                    message.body = body;
                    message.is_edited = true
                }
                MessageLikePayload::DeliveryReceipt => message.is_delivered = true,
                MessageLikePayload::ReadReceipt => message.is_read = true,
                MessageLikePayload::Message { .. } => unreachable!(),
                MessageLikePayload::Reaction { mut emojis } => {
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
                MessageLikePayload::Retraction => {
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

    use prose_xmpp::bare;

    use crate::domain::messaging::models::{MessageLike, MessageLikePayload};
    use crate::test::MessageBuilder;

    use super::*;

    #[test]
    fn test_toggle_reaction() {
        let mut message = MessageBuilder::new_with_index(1).build_message();
        assert!(message.reactions.is_empty());

        message.toggle_reaction(&bare!("a@prose.org"), "🎉".into());
        assert_eq!(
            message.reactions,
            vec![Reaction {
                emoji: "🎉".into(),
                from: vec![bare!("a@prose.org")]
            }]
        );

        message.toggle_reaction(&bare!("b@prose.org"), "🎉".into());
        assert_eq!(
            message.reactions,
            vec![Reaction {
                emoji: "🎉".into(),
                from: vec![bare!("a@prose.org"), bare!("b@prose.org")]
            }]
        );

        message.toggle_reaction(&bare!("b@prose.org"), "✅".into());
        assert_eq!(
            message.reactions,
            vec![
                Reaction {
                    emoji: "🎉".into(),
                    from: vec![bare!("a@prose.org"), bare!("b@prose.org")]
                },
                Reaction {
                    emoji: "✅".into(),
                    from: vec![bare!("b@prose.org")]
                }
            ]
        );

        message.toggle_reaction(&bare!("a@prose.org"), "🎉".into());
        assert_eq!(
            message.reactions,
            vec![
                Reaction {
                    emoji: "🎉".into(),
                    from: vec![bare!("b@prose.org")]
                },
                Reaction {
                    emoji: "✅".into(),
                    from: vec![bare!("b@prose.org")]
                }
            ]
        );

        message.toggle_reaction(&bare!("b@prose.org"), "🎉".into());
        assert_eq!(
            message.reactions,
            vec![
                Reaction {
                    emoji: "🎉".into(),
                    from: vec![]
                },
                Reaction {
                    emoji: "✅".into(),
                    from: vec![bare!("b@prose.org")]
                }
            ]
        );
    }

    #[test]
    fn test_reactions_for_user() {
        let mut message = MessageBuilder::new_with_index(1).build_message();
        message.reactions = vec![
            Reaction {
                emoji: "🎉".into(),
                from: vec![bare!("a@prose.org"), bare!("b@prose.org")],
            },
            Reaction {
                emoji: "✅".into(),
                from: vec![bare!("b@prose.org")],
            },
        ];

        assert_eq!(
            message
                .reactions_from(&bare!("a@prose.org"))
                .cloned()
                .collect::<Vec<Emoji>>(),
            vec!["🎉".into()]
        );
        assert_eq!(
            message
                .reactions_from(&bare!("b@prose.org"))
                .cloned()
                .collect::<Vec<Emoji>>(),
            vec!["🎉".into(), "✅".into()]
        );
    }

    #[test]
    fn test_reduces_emojis() {
        let messages = [
            MessageLike {
                id: "1".into(),
                stanza_id: None,
                target: None,
                to: Some(BareJid::from_str("a@prose.org").unwrap()),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 00)
                    .unwrap()
                    .into(),
                payload: MessageLikePayload::Message {
                    body: String::from("Hello World"),
                },
                is_first_message: false,
            },
            MessageLike {
                id: "2".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: Some(BareJid::from_str("a@prose.org").unwrap()),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 01)
                    .unwrap()
                    .into(),
                payload: MessageLikePayload::Reaction {
                    emojis: vec!["👍".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "3".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: Some(BareJid::from_str("a@prose.org").unwrap()),
                from: BareJid::from_str("c@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 02)
                    .unwrap()
                    .into(),
                payload: MessageLikePayload::Reaction {
                    emojis: vec!["👍".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "4".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: Some(BareJid::from_str("a@prose.org").unwrap()),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 03)
                    .unwrap()
                    .into(),
                payload: MessageLikePayload::Reaction {
                    emojis: vec!["👍".into(), "📼".into(), "🍿".into(), "☕️".into()],
                },
                is_first_message: false,
            },
            MessageLike {
                id: "5".into(),
                stanza_id: None,
                target: Some("1".into()),
                to: Some(BareJid::from_str("a@prose.org").unwrap()),
                from: BareJid::from_str("b@prose.org").unwrap(),
                timestamp: Utc
                    .with_ymd_and_hms(2023, 04, 07, 16, 00, 04)
                    .unwrap()
                    .into(),
                payload: MessageLikePayload::Reaction {
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
