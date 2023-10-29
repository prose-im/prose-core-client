// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use crate::domain::shared::models::Availability;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Presence {
    pub priority: i8,
    pub availability: Availability,
    pub status: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_unavailable() {
        assert_eq!(
            Presence::default(),
            Presence {
                priority: 0,
                availability: Availability::Unavailable,
                status: None,
            }
        )
    }
}
