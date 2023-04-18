use jid::BareJid;
use strum_macros::{Display, EnumString};

use crate::helpers::StanzaCow;
use crate::stanza_base;

#[derive(Debug, PartialEq, Display, EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Subscription {
    None,
    To,
    From,
    Both,
}

#[derive(Debug, PartialEq, Display, EnumString, Clone)]
#[strum(serialize_all = "lowercase")]
pub enum Ask {
    Subscribe,
}

pub struct Item<'a> {
    stanza: StanzaCow<'a>,
}

impl<'a> Item<'a> {
    pub fn name(&self) -> Option<&str> {
        self.attribute("name")
    }

    pub fn jid(&self) -> Option<BareJid> {
        self.attribute("jid")
            .and_then(|s| s.parse::<BareJid>().ok())
    }

    pub fn subscription(&self) -> Subscription {
        self.attribute("subscription")
            .and_then(|s| s.parse::<Subscription>().ok())
            .unwrap_or(Subscription::None)
    }

    pub fn ask(&self) -> Option<Ask> {
        self.attribute("ask").and_then(|s| s.parse::<Ask>().ok())
    }

    pub fn groups(&self) -> Vec<String> {
        let mut groups = vec![];
        for child in self.stanza.children() {
            if child.name() != Some("group") {
                continue;
            }
            if let Some(name) = child.text() {
                groups.push(name);
            }
        }
        groups
    }
}

stanza_base!(Item);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_items() {
        let item1 =
            Item::from_str(r#"<item ask="subscribe" jid="remi@prose.org" subscription="none"/>"#)
                .unwrap();
        assert_eq!(
            item1.jid(),
            Some(BareJid::from_str("remi@prose.org").unwrap())
        );
        assert_eq!(item1.ask(), Some(Ask::Subscribe));
        assert_eq!(item1.subscription(), Subscription::None);
        assert_eq!(item1.groups(), Vec::<String>::new());

        let item2 =
            Item::from_str(r#"<item jid="valerian@valeriansaliou.name" subscription="both"><group>Contacts</group><group>Buddies</group></item>"#)
                .unwrap();
        assert_eq!(
            item2.jid(),
            Some(BareJid::from_str("valerian@valeriansaliou.name").unwrap())
        );
        assert_eq!(item2.ask(), None);
        assert_eq!(item2.subscription(), Subscription::Both);
        assert_eq!(
            item2.groups(),
            vec!["Contacts".to_string(), "Buddies".to_string()]
        );

        let item3 = Item::from_str(r#"<item jid="a@prose.org"/>"#).unwrap();
        assert_eq!(item3.jid(), Some(BareJid::from_str("a@prose.org").unwrap()));
        assert_eq!(item3.ask(), None);
        assert_eq!(item3.subscription(), Subscription::None);
        assert_eq!(item3.groups(), Vec::<String>::new());
    }
}
