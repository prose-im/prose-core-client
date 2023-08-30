use crate::ns;
use crate::util::ElementExt;
use minidom::{Element, NSChoice};
use std::str::FromStr;
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::iq::{IqGetPayload, IqSetPayload};

pub enum Role {
    Owner,
    Admin,
}

pub struct Query {
    pub role: Role,
    pub payloads: Vec<Element>,
}

impl Query {
    pub fn new(role: Role) -> Self {
        Query {
            role,
            payloads: Default::default(),
        }
    }

    pub fn with_payload(mut self, payload: impl MucQueryPayload) -> Self {
        self.payloads.push(payload.into());
        self
    }

    pub fn with_payloads(mut self, payloads: Vec<Element>) -> Self {
        self.payloads = payloads;
        self
    }
}

impl From<Query> for Element {
    fn from(value: Query) -> Self {
        Element::builder("query", value.role.to_string())
            .append_all(value.payloads)
            .build()
    }
}

impl TryFrom<Element> for Query {
    type Error = anyhow::Error;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("query", NSChoice::AnyOf(&[ns::MUC_OWNER, ns::MUC_ADMIN]))?;

        let payloads = root
            .children()
            .into_iter()
            .map(|child| match child {
                _ if child.is("item", ns::MUC_USER) => Ok(child.clone()),
                _ if child.is("x", ns::DATA_FORMS) => Ok(child.clone()),
                _ => Err(anyhow::format_err!(
                    "Encountered unexpected payload {} in muc query.",
                    child.name()
                )),
            })
            .collect::<Result<Vec<Element>, _>>()?;

        Ok(Query {
            role: Role::from_str(&root.ns())?,
            payloads,
        })
    }
}

impl IqSetPayload for Query {}
impl IqGetPayload for Query {}

pub trait MucQueryPayload: TryFrom<Element> + Into<Element> {}

impl MucQueryPayload for DataForm {}
impl MucQueryPayload for xmpp_parsers::muc::user::Item {}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ns::MUC_OWNER => Ok(Self::Owner),
            ns::MUC_ADMIN => Ok(Self::Admin),
            _ => Err(anyhow::format_err!("Unknown role {}", s)),
        }
    }
}

impl ToString for Role {
    fn to_string(&self) -> String {
        match self {
            Self::Owner => ns::MUC_OWNER,
            Self::Admin => ns::MUC_ADMIN,
        }
        .to_string()
    }
}
