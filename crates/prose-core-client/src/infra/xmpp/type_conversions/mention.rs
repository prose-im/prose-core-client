// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::ops::Range;

use anyhow::{anyhow, bail, Result};

use prose_xmpp::stanza::references::{Reference, ReferenceType};

use crate::domain::messaging::models::Mention;
use crate::domain::shared::models::UserId;
use crate::dtos::UnicodeScalarIndex;

impl From<Mention> for Reference {
    fn from(value: Mention) -> Self {
        let mut reference = Self::mention(value.user.into_inner());
        reference.begin = Some(*value.range.start.as_ref());
        reference.end = Some(*value.range.end.as_ref());
        reference
    }
}

impl TryFrom<Reference> for Mention {
    type Error = anyhow::Error;

    fn try_from(value: Reference) -> Result<Self> {
        if value.r#type != ReferenceType::Mention {
            bail!("Invalid reference type '{:?}'", value.r#type)
        }

        Ok(Self {
            user: UserId::from_iri(&value.uri)?,
            range: Range {
                start: UnicodeScalarIndex::new(
                    value.begin.ok_or(anyhow!("Missing 'begin' in Reference"))?,
                ),
                end: UnicodeScalarIndex::new(
                    value.end.ok_or(anyhow!("Missing 'end' in Reference"))?,
                ),
            },
        })
    }
}
