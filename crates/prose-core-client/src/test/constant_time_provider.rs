// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeZone, Utc};

use prose_xmpp::TimeProvider;

pub struct ConstantTimeProvider {
    pub time: DateTime<Utc>,
}

impl ConstantTimeProvider {
    pub fn ymd(year: i32, month: u32, day: u32) -> Self {
        ConstantTimeProvider {
            time: Utc
                .with_ymd_and_hms(year, month, day, 0, 0, 0)
                .unwrap()
                .into(),
        }
    }
}

impl TimeProvider for ConstantTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        self.time.clone()
    }
}
