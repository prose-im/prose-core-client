use minidom::Element;

use crate::stanza::message;
use crate::util::id_string_macro::id_string;

id_string!(Emoji);

#[derive(Debug, PartialEq, Clone)]
pub struct Reactions {
    pub id: message::Id,
    pub reactions: Vec<Emoji>,
}

impl TryFrom<Element> for Reactions {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let reactions = xmpp_parsers::reactions::Reactions::try_from(value)?;
        Ok(Reactions {
            id: reactions.id.into(),
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
            id: value.id.into_inner(),
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
