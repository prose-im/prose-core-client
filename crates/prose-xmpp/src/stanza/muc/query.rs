use crate::ns;
use crate::util::{ElementExt, RequestError};
use jid::{BareJid, Jid};
use minidom::{Element, NSChoice};
use std::str::FromStr;
use xmpp_parsers::data_forms::DataForm;
use xmpp_parsers::iq::{IqGetPayload, IqSetPayload};
use xmpp_parsers::muc;

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    Owner,
    Admin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub role: Role,
    pub payloads: Vec<Element>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Destroy {
    pub jid: Option<BareJid>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub jid: Jid,
    pub affiliation: muc::user::Affiliation,
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
    type Error = RequestError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("query", NSChoice::AnyOf(&[ns::MUC_OWNER, ns::MUC_ADMIN]))?;

        let payloads = root
            .children()
            .into_iter()
            .map(|child| match child {
                _ if child.is("item", NSChoice::AnyOf(&[ns::MUC_OWNER, ns::MUC_ADMIN])) => {
                    Ok(child.clone())
                }
                _ if child.is("x", ns::DATA_FORMS) => Ok(child.clone()),
                _ => Err(RequestError::Generic {
                    msg: format!(
                        "Encountered unexpected payload {} in muc query.",
                        child.name()
                    ),
                }),
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
impl MucQueryPayload for Destroy {}

impl FromStr for Role {
    type Err = RequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ns::MUC_OWNER => Ok(Self::Owner),
            ns::MUC_ADMIN => Ok(Self::Admin),
            _ => Err(RequestError::Generic {
                msg: format!("Unknown role {}", s),
            }),
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

impl TryFrom<Element> for Destroy {
    type Error = RequestError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("destroy", ns::MUC_OWNER)?;

        Ok(Destroy {
            jid: root.attr("jid").map(BareJid::from_str).transpose()?,
            reason: root
                .get_child("destroy", ns::MUC_OWNER)
                .map(|node| node.text()),
        })
    }
}

impl From<Destroy> for Element {
    fn from(value: Destroy) -> Self {
        Element::builder("destroy", ns::MUC_OWNER)
            .attr("jid", value.jid)
            .append_all(value.reason.map(|reason| {
                Element::builder("reason", ns::MUC_OWNER)
                    .append(reason)
                    .build()
            }))
            .build()
    }
}

impl TryFrom<Element> for User {
    type Error = RequestError;

    fn try_from(root: Element) -> Result<Self, Self::Error> {
        root.expect_is("item", NSChoice::AnyOf(&[ns::MUC_OWNER, ns::MUC_ADMIN]))?;

        Ok(User {
            jid: Jid::from_str(root.attr_req("jid")?)?,
            affiliation: muc::user::Affiliation::from_str(root.attr_req("affiliation")?).map_err(
                |err| RequestError::Generic {
                    msg: err.to_string(),
                },
            )?,
        })
    }
}
