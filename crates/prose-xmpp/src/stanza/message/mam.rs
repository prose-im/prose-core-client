// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use minidom::Element;
pub use xmpp_parsers::mam::{Fin, Query, QueryId};
use xmpp_parsers::message::MessagePayload;

use crate::stanza::message::{stanza_id, Forwarded};

/// The wrapper around forwarded stanzas.
#[derive(Debug, PartialEq, Clone)]
pub struct ArchivedMessage {
    pub id: stanza_id::Id,
    pub query_id: Option<QueryId>,
    pub forwarded: Forwarded,
}

impl TryFrom<Element> for ArchivedMessage {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        let result = xmpp_parsers::mam::Result_::try_from(value)?;
        Ok(ArchivedMessage {
            id: result.id.into(),
            query_id: result.queryid,
            forwarded: result.forwarded.try_into()?,
        })
    }
}

impl From<ArchivedMessage> for Element {
    fn from(value: ArchivedMessage) -> Self {
        xmpp_parsers::mam::Result_ {
            id: value.id.into_inner(),
            queryid: value.query_id,
            forwarded: value.forwarded.into(),
        }
        .into()
    }
}

impl MessagePayload for ArchivedMessage {}
