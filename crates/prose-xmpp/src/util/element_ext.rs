// prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use anyhow::Result;
use minidom::{Element, NSChoice};

pub trait ElementExt {
    fn expect_is<'a>(&self, name: impl AsRef<str>, ns: impl Into<NSChoice<'a>>) -> Result<()>;

    fn req_attr(&self, name: impl AsRef<str>) -> Result<&str>;
}

impl ElementExt for Element {
    fn expect_is<'a>(&self, name: impl AsRef<str>, ns: impl Into<NSChoice<'a>>) -> Result<()> {
        let ns = ns.into();
        if !self.is(&name, ns) {
            return Err(anyhow::format_err!(
                "Expected element with name {} and namespace {}. Got {} and {} instead.",
                name.as_ref(),
                ns_choice_to_string(ns),
                self.name(),
                self.ns()
            ));
        }
        Ok(())
    }

    fn req_attr(&self, name: impl AsRef<str>) -> Result<&str> {
        self.attr(name.as_ref()).ok_or(anyhow::format_err!(
            "Missing required attribute {} in element {}.",
            name.as_ref(),
            self.name()
        ))
    }
}

fn ns_choice_to_string<'a>(ns: impl Into<NSChoice<'a>>) -> String {
    match ns.into() {
        NSChoice::None => "<none>".to_string(),
        NSChoice::OneOf(ns) => ns.to_string(),
        NSChoice::AnyOf(ns_list) => ns_list.join(" or "),
        NSChoice::Any => "<any>".to_string(),
    }
}
