// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use prose_xmpp::stanza::http_upload::OOB;

use crate::domain::messaging::models::Attachment;

impl From<Attachment> for OOB {
    fn from(value: Attachment) -> Self {
        OOB {
            url: value.url.to_string(),
            desc: value.description,
        }
    }
}

impl TryFrom<OOB> for Attachment {
    type Error = anyhow::Error;

    fn try_from(value: OOB) -> Result<Self, Self::Error> {
        Ok(Attachment {
            url: value.url.parse()?,
            description: value.desc,
        })
    }
}
