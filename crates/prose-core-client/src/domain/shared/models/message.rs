// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use prose_markup::MarkdownParser;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// A message in Markdown format.
pub struct Markdown(String);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// A HTML message.
pub struct HTML(String);

#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
/// A message styled according to XEP-0393: Message Styling.
pub struct StyledMessage(String);

impl Markdown {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn to_html(&self) -> HTML {
        let parser = MarkdownParser::new(&self.0);
        HTML(parser.convert_to_html())
    }
}

impl HTML {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl StyledMessage {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn into_html(self) -> HTML {
        // We're not parsing the message styling format yet.
        HTML(format!(
            "<p>{}</p>",
            self.0.lines().collect::<Vec<_>>().join("<br/>")
        ))
    }
}

impl Display for Markdown {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T> From<T> for Markdown
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for Markdown {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for HTML {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T> From<T> for HTML
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for HTML {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for StyledMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<T> From<T> for StyledMessage
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for StyledMessage {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
