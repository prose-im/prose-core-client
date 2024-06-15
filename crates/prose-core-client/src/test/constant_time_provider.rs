// prose-core-client/prose-core-client
//
// Copyright: 2023, Marc Bauer <mb@nesium.com>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use parking_lot::Mutex;

use prose_xmpp::TimeProvider;

#[derive(Clone)]
pub struct ConstantTimeProvider {
    pub time: Arc<Mutex<DateTime<Utc>>>,
}

impl Default for ConstantTimeProvider {
    fn default() -> Self {
        Self::new(Utc::now())
    }
}

impl ConstantTimeProvider {
    pub fn new(time: DateTime<Utc>) -> Self {
        Self {
            time: Arc::new(Mutex::new(time)),
        }
    }

    pub fn ymd(year: i32, month: u32, day: u32) -> Self {
        Self::ymd_hms(year, month, day, 0, 0, 0)
    }

    pub fn ymd_hms(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Self {
        ConstantTimeProvider {
            time: Arc::new(Mutex::new(
                Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
                    .unwrap(),
            )),
        }
    }

    pub fn set_ymd(&self, year: i32, month: u32, day: u32) {
        self.set_ymd_hms(year, month, day, 0, 0, 0);
    }

    pub fn set_ymd_hms(&self, year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) {
        self.set_ymd_hms_millis(year, month, day, hour, min, sec, 0)
    }

    pub fn set_ymd_hms_millis(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        milli: u32,
    ) {
        *self.time.lock() = Utc.from_utc_datetime(
            &NaiveDate::from_ymd_opt(year, month, day)
                .unwrap()
                .and_hms_milli_opt(hour, min, sec, milli)
                .unwrap(),
        )
    }
}

impl TimeProvider for ConstantTimeProvider {
    fn now(&self) -> DateTime<Utc> {
        self.time.lock().clone()
    }
}
