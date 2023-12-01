// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Represents the identifier of a request directed at us.
pub struct RequestId(String);

impl<T> From<T> for RequestId
where
    T: Into<String>,
{
    fn from(s: T) -> Self {
        RequestId(s.into())
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
