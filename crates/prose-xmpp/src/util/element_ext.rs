// prose-core-client/prose-xmpp
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::util::ParseError;
use minidom::{Element, ElementBuilder, NSChoice};

pub trait ElementExt {
    fn expect_is<'a>(
        &self,
        name: impl AsRef<str>,
        ns: impl Into<NSChoice<'a>>,
    ) -> Result<(), ParseError>;

    fn attr_req(&self, name: impl AsRef<str>) -> Result<&str, ParseError>;

    fn attr_bool(&self, name: impl AsRef<str>) -> Result<Option<bool>, ParseError>;
    fn attr_bool_req(&self, name: impl AsRef<str>) -> Result<bool, ParseError>;

    fn non_empty_text(&self) -> Option<String>;
}

pub trait ElementBuilderExt {
    fn attr_bool(self, name: impl AsRef<str>, value: bool) -> ElementBuilder;
    fn attr_bool_opt(self, name: impl AsRef<str>, value: Option<bool>) -> ElementBuilder;
}

impl ElementExt for Element {
    fn expect_is<'a>(
        &self,
        name: impl AsRef<str>,
        ns: impl Into<NSChoice<'a>>,
    ) -> Result<(), ParseError> {
        let ns = ns.into();
        if !self.is(&name, ns) {
            return Err(ParseError::Generic {
                msg: format!(
                    "Expected element with name {} and namespace {}. Got {} and {} instead.",
                    name.as_ref(),
                    ns_choice_to_string(ns),
                    self.name(),
                    self.ns()
                ),
            });
        }
        Ok(())
    }

    fn attr_req(&self, name: impl AsRef<str>) -> Result<&str, ParseError> {
        self.attr(name.as_ref()).ok_or(ParseError::Generic {
            msg: format!(
                "Missing required attribute {} in element {}.",
                name.as_ref(),
                self.name()
            ),
        })
    }

    fn attr_bool(&self, name: impl AsRef<str>) -> Result<Option<bool>, ParseError> {
        self.attr(name.as_ref()).map(parse_bool).transpose()
    }

    fn attr_bool_req(&self, name: impl AsRef<str>) -> Result<bool, ParseError> {
        parse_bool(self.attr_req(name)?)
    }

    fn non_empty_text(&self) -> Option<String> {
        let text = self.text();
        (!text.is_empty()).then_some(text)
    }
}

impl ElementBuilderExt for ElementBuilder {
    fn attr_bool(self, name: impl AsRef<str>, value: bool) -> ElementBuilder {
        self.attr(name.as_ref(), if value { "true" } else { "false" })
    }

    fn attr_bool_opt(self, name: impl AsRef<str>, value: Option<bool>) -> ElementBuilder {
        let Some(true) = value else { return self };
        self.attr_bool(name, true)
    }
}

pub fn parse_bool(value: impl AsRef<str>) -> Result<bool, ParseError> {
    Ok(match value.as_ref() {
        "true" | "1" => true,
        "false" | "0" => false,
        _ => {
            return Err(ParseError::Generic {
                msg: format!("Unknown value '{}' 'continue' attribute", value.as_ref()),
            })
        }
    })
}

fn ns_choice_to_string<'a>(ns: impl Into<NSChoice<'a>>) -> String {
    match ns.into() {
        NSChoice::None => "<none>".to_string(),
        NSChoice::OneOf(ns) => ns.to_string(),
        NSChoice::AnyOf(ns_list) => ns_list.join(" or "),
        NSChoice::Any => "<any>".to_string(),
    }
}
