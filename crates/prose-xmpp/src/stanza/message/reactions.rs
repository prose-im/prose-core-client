// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use prose_utils::id_string;

id_string!(Emoji);

#[derive(Debug, PartialEq, Clone)]
pub struct Reactions {
    pub id: String,
    pub reactions: Vec<Emoji>,
}

impl TryFrom<Element> for Reactions {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let reactions = xmpp_parsers::reactions::Reactions::try_from(value)?;
        Ok(Reactions {
            id: reactions.id,
            reactions: reactions
                .reactions
                .into_iter()
                .map(|r| r.emoji.into())
                .collect(),
        })
    }
}

impl From<Reactions> for Element {
    fn from(value: Reactions) -> Self {
        xmpp_parsers::reactions::Reactions {
            id: value.id,
            reactions: value
                .reactions
                .into_iter()
                .map(|r| xmpp_parsers::reactions::Reaction {
                    emoji: r.into_inner(),
                })
                .collect(),
        }
        .into()
    }
}

impl MessagePayload for Reactions {}
