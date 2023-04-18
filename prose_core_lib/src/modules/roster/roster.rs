use crate::modules::{roster, Context, Module};
use crate::stanza::iq::Kind;
use crate::stanza::{Namespace, Stanza, StanzaBase, IQ};

pub struct Roster {}

impl Roster {
    pub fn new() -> Self {
        Roster {}
    }
}

impl Module for Roster {}

impl Roster {
    pub async fn load_roster(&self, ctx: &Context<'_>) -> anyhow::Result<Vec<roster::types::Item>> {
        let iq = IQ::new(Kind::Get, ctx.generate_id())
            .add_child(Stanza::new_query(Namespace::Roster, None));
        Ok(ctx.send_iq(iq).await?.roster_items())
    }
}

impl<'a> IQ<'a> {
    fn roster_items<'b>(&self) -> Vec<roster::types::Item<'b>> {
        let Some(roster_stanza) = self.child_by_name_and_namespace("query", Namespace::Roster) else {
            return vec![]
        };

        roster_stanza
            .children()
            .filter_map(|child| {
                if child.name() != Some("item") {
                    return None;
                }
                let item: roster::types::Item = child.into();
                Some(item.clone())
            })
            .collect()
    }
}
