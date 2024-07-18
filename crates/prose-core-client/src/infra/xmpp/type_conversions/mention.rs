// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::{bail, Result};

use prose_xmpp::stanza::references::{Reference, ReferenceType};

use crate::domain::messaging::models::Mention;
use crate::domain::shared::models::UserId;
use crate::dtos::UnicodeScalarIndex;

impl From<Mention> for Reference {
    fn from(value: Mention) -> Self {
        let mut reference = Self::mention(value.user.into_inner());
        reference.begin = value.range.as_ref().map(|r| *r.start.as_ref());
        reference.end = value.range.as_ref().map(|r| *r.end.as_ref());
        reference
    }
}

impl TryFrom<Reference> for Mention {
    type Error = anyhow::Error;

    fn try_from(value: Reference) -> Result<Self> {
        if value.r#type != ReferenceType::Mention {
            bail!("Invalid reference type '{:?}'", value.r#type)
        }

        let range = value.begin.and_then(|begin| {
            value
                .end
                .map(|end| UnicodeScalarIndex::new(begin)..UnicodeScalarIndex::new(end))
        });

        Ok(Self {
            user: UserId::from_iri(&value.uri)?,
            range,
        })
    }
}
