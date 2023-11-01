// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use chrono::{DateTime, TimeZone, Utc};
use parking_lot::Mutex;

use prose_xmpp::TimeProvider;

pub struct ConstantTimeProvider {
    pub time: Mutex<DateTime<Utc>>,
}

impl ConstantTimeProvider {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self {
            time: Mutex::new(time),
        }
    }

    pub fn ymd(year: i32, month: u32, day: u32) -> Self {
        Self::ymd_hms(year, month, day, 0, 0, 0)
    }

    pub fn ymd_hms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Self {
        ConstantTimeProvider {
            time: Mutex::new(
                Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
                    .unwrap(),
            ),
        }
    }

    pub fn set_ymd(&self, year: i32, month: u32, day: u32) {
        self.set_ymd_hms(year, month, day, 0, 0, 0);
    }

    pub fn set_ymd_hms(&self, year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) {
        *self.time.lock() = Utc
            .with_ymd_and_hms(year, month, day, hour, min, sec)
            .unwrap()
    }
}

impl TimeProvider for ConstantTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        self.time.lock().clone()
    }
}
