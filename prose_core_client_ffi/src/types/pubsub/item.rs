use crate::error::{Error, StanzaParseError};
use crate::helpers::StanzaExt;
use libstrophe::Stanza;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
pub struct Item<T> {
    pub id: Option<String>,
    pub value: T,
}

impl<T> Item<T> {
    pub fn new(id: Option<String>, value: T) -> Self {
        Self { id, value }
    }
}

impl<T> TryFrom<&Stanza> for Item<T>
where
    for<'a> T: TryFrom<&'a Stanza, Error = Error> + Sized,
{
    type Error = Error;

    fn try_from(stanza: &Stanza) -> Result<Self, Self::Error> {
        let first_child = match stanza.get_first_non_text_child() {
            Some(id) => id,
            None => {
                return Err(Error::StanzaParseError {
                    error: StanzaParseError::missing_child_node("<any>", stanza),
                })
            }
        };

        let id = stanza.id().map(|s| s.to_string());
        let value = T::try_from(first_child.deref())?;

        return Ok(Item::new(id, value));
    }
}
