// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::Element;
use xmpp_parsers::iq::{Iq, IqGetPayload};

use crate::client::ModuleContext;
use crate::mods::Module;
use crate::ns;
use crate::util::{ElementExt, RequestError};

#[derive(Default, Clone)]
pub struct Roster {
    ctx: ModuleContext,
}

impl Module for Roster {
    fn register_with(&mut self, context: ModuleContext) {
        self.ctx = context
    }
}

impl Roster {
    pub async fn load_roster(&self) -> Result<xmpp_parsers::roster::Roster> {
        let roster = self
            .ctx
            .send_iq(Iq::from_get(
                self.ctx.generate_id(),
                Query::new(self.ctx.generate_id()),
            ))
            .await?;

        let Some(response) = roster else {
            return Err(RequestError::UnexpectedResponse.into());
        };

        Ok(xmpp_parsers::roster::Roster::try_from(response)?)
    }
}

struct Query {
    query_id: String,
}

impl Query {
    fn new(query_id: impl Into<String>) -> Self {
        Query {
            query_id: query_id.into(),
        }
    }
}

impl From<Query> for Element {
    fn from(value: Query) -> Self {
        Element::builder("query", ns::ROSTER)
            .attr("queryid", value.query_id)
            .build()
    }
}

impl TryFrom<Element> for Query {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Query {
            query_id: value.req_attr("queryid")?.to_string(),
        })
    }
}

impl IqGetPayload for Query {}
