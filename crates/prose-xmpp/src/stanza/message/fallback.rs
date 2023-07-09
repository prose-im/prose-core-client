use minidom::Element;
use xmpp_parsers::message::MessagePayload;

use crate::ns;
use crate::util::ElementExt;

/// XEP-0428
#[derive(Debug, PartialEq, Clone)]
pub struct Fallback {
    pub r#for: Option<String>,
    pub subjects: Vec<Range>,
    pub bodies: Vec<Range>,
}

impl Fallback {
    pub fn new() -> Self {
        Fallback {
            r#for: None,
            subjects: vec![],
            bodies: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Range {
    pub start: Option<i32>,
    pub end: Option<i32>,
}

impl TryFrom<Element> for Fallback {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        value.expect_is("fallback", ns::FALLBACK)?;

        let mut fallback = Fallback {
            r#for: value.attr("for").map(str::to_string),
            subjects: vec![],
            bodies: vec![],
        };

        for child in value.children() {
            match child.name() {
                "subject" => fallback.subjects.push(Range::try_from(child.clone())?),
                "body" => fallback.bodies.push(Range::try_from(child.clone())?),
                _ => (),
            }
        }

        Ok(fallback)
    }
}

impl From<Fallback> for Element {
    fn from(value: Fallback) -> Self {
        Element::builder("fallback", ns::FALLBACK)
            .attr("for", value.r#for)
            .append_all(value.subjects.into_iter().map(|range| {
                Element::builder("subject", ns::FALLBACK)
                    .attr("start", range.start)
                    .attr("end", range.end)
                    .build()
            }))
            .append_all(value.bodies.into_iter().map(|range| {
                Element::builder("body", ns::FALLBACK)
                    .attr("start", range.start)
                    .attr("end", range.end)
                    .build()
            }))
            .build()
    }
}

impl TryFrom<Element> for Range {
    type Error = anyhow::Error;

    fn try_from(value: Element) -> Result<Self, Self::Error> {
        Ok(Range {
            start: value
                .attr("start")
                .map(|start| i32::from_str_radix(&start, 10))
                .transpose()?,
            end: value
                .attr("end")
                .map(|start| i32::from_str_radix(&start, 10))
                .transpose()?,
        })
    }
}

impl MessagePayload for Fallback {}
