// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Copy)]
pub enum Availability {
    Available,
    #[default]
    Unavailable,
    DoNotDisturb,
    Away,
}

impl Display for Availability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Availability::Available => "available",
                Availability::Unavailable => "unavailable",
                Availability::DoNotDisturb => "do not disturb",
                Availability::Away => "away",
            }
        )
    }
}
