// prose-core-client/prose-core-client
//
// Copyright: 2024, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AvatarId(String);

impl AvatarId {
    pub fn from_str_unchecked(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl FromStr for AvatarId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(is_valid_sha1(s), "Avatar ID is not a valid SHA1 hash.");
        Ok(Self(s.to_string()))
    }
}

impl Display for AvatarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn is_valid_sha1(s: &str) -> bool {
    s.len() == 40 && s.chars().all(|c| c.is_digit(16))
}
