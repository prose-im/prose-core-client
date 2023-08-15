use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use prose_xmpp::TimeProvider;

pub struct ConstantTimeProvider {
    pub time: DateTime<FixedOffset>,
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
    fn now(&self) -> DateTime<FixedOffset> {
        self.time.clone()
    }
}
