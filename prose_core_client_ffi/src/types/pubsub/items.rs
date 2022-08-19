use super::Item;
use crate::error::Error;
use libstrophe::Stanza;
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq)]
pub struct Items<T>(Vec<Item<T>>);

impl<T> IntoIterator for Items<T> {
    type Item = Item<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Deref for Items<T> {
    type Target = Vec<Item<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Items<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> TryFrom<&Stanza> for Items<T>
where
    for<'a> T: TryFrom<&'a Stanza, Error = Error> + Sized,
{
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let items: Result<Vec<_>, _> = stanza
            .children()
            .filter(|n| n.name() == Some("item"))
            .map(|n| Item::try_from(n.deref()))
            .collect();

        Ok(Items(items?))
    }
}

#[cfg(test)]
mod tests {
    use crate::types::stanza_id::StanzaID;
    use jid::BareJid;
    use libstrophe::Stanza;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_deserializes_items() {
        let xml = r#"
        <items>
            <item id="i1"><stanza-id id="s1" by="a@prose.org"/></item>
            <item id="i2"><stanza-id id="s2" by="b@prose.org"/></item>
            <item id="i3"><stanza-id id="s3" by="c@prose.org"/></item>
        </items>
        "#;

        let stanza = Stanza::from_str(xml);
        let items = Items::<StanzaID>::try_from(&stanza).unwrap();

        assert_eq!(
            items,
            Items(vec![
                Item::new(
                    Some("i1".to_string()),
                    StanzaID::new("s1", BareJid::from_str("a@prose.org").unwrap())
                ),
                Item::new(
                    Some("i2".to_string()),
                    StanzaID::new("s2", BareJid::from_str("b@prose.org").unwrap())
                ),
                Item::new(
                    Some("i3".to_string()),
                    StanzaID::new("s3", BareJid::from_str("c@prose.org").unwrap())
                )
            ])
        );
    }

    #[test]
    fn test_deserializes_empty_items() {
        let xml = r#"<items />"#;

        let stanza = Stanza::from_str(xml);
        let items = Items::<StanzaID>::try_from(&stanza).unwrap();

        assert_eq!(items, Items(vec![]));
    }
}
