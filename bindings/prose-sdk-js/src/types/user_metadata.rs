use chrono::{FixedOffset, Utc};
use prose_core_client::types::user_metadata;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct UserMetadata(prose_core_client::types::UserMetadata);

#[wasm_bindgen]
pub struct LastActivity(user_metadata::LastActivity);

#[wasm_bindgen]
pub struct DateTime(chrono::DateTime<FixedOffset>);

#[wasm_bindgen]
#[derive(Default)]
pub struct UserVerification {
    fingerprint: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    identity: Option<String>,
}

#[wasm_bindgen]
#[derive(Default)]
pub struct UserEncryption {
    secure_protocol: bool,
    connection_protocol: Option<String>,
    end_to_end_method: Option<String>,
}

impl From<prose_core_client::types::UserMetadata> for UserMetadata {
    fn from(value: prose_core_client::types::UserMetadata) -> Self {
        UserMetadata(value)
    }
}

#[wasm_bindgen]
impl UserMetadata {
    /// The local time of the user
    #[wasm_bindgen(getter, js_name = "localTime")]
    pub fn local_time(&self) -> Option<DateTime> {
        self.0.local_time.map(|t| DateTime(t.clone()))
    }

    /// The last activity of the user
    #[wasm_bindgen(getter, js_name = "lastActivity")]
    pub fn last_activity(&self) -> Option<LastActivity> {
        self.0
            .last_activity
            .as_ref()
            .map(|a| LastActivity(a.clone()))
    }

    /// Not implemented yet
    #[wasm_bindgen(getter)]
    pub fn verification(&self) -> Option<UserVerification> {
        None
    }

    /// Not implemented yet
    #[wasm_bindgen(getter)]
    pub fn encryption(&self) -> Option<UserEncryption> {
        None
    }
}

#[wasm_bindgen]
impl LastActivity {
    /// The time when the user was last active in seconds since January 1, 1970 0:00:00 UTC
    #[wasm_bindgen(getter, js_name = "utcTimestamp")]
    pub fn utc_timestamp(&self) -> i64 {
        self.0.timestamp.timestamp()
    }

    /// The status they sent when setting their presence to 'unavailable'.
    #[wasm_bindgen(getter)]
    pub fn status(&self) -> Option<String> {
        self.0.status.clone()
    }
}

#[wasm_bindgen]
impl DateTime {
    /// The number of non-leap seconds since January 1, 1970 0:00:00 UTC (aka "UNIX timestamp").
    #[wasm_bindgen(getter, js_name = "timestamp")]
    pub fn timestamp(&self) -> i64 {
        self.0.with_timezone(&Utc).timestamp()
    }

    /// The number of seconds to add to convert from UTC to the local time.
    #[wasm_bindgen(getter, js_name = "timezoneOffset")]
    pub fn offset(&self) -> i32 {
        self.0.offset().local_minus_utc()
    }

    #[wasm_bindgen(getter, js_name = "formattedTimezoneOffset")]
    pub fn formatted_timezone_offset(&self) -> String {
        format!("{}", self.0.timezone())
    }
}

#[wasm_bindgen]
impl UserVerification {
    #[wasm_bindgen(getter)]
    pub fn fingerprint(&self) -> Option<String> {
        self.fingerprint.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn email(&self) -> Option<String> {
        self.email.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn phone(&self) -> Option<String> {
        self.phone.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> Option<String> {
        self.identity.clone()
    }
}

#[wasm_bindgen]
impl UserEncryption {
    #[wasm_bindgen(getter, js_name = "secureProtocol")]
    pub fn secure_protocol(&self) -> bool {
        self.secure_protocol
    }

    #[wasm_bindgen(getter, js_name = "connectionProtocol")]
    pub fn connection_protocol(&self) -> Option<String> {
        self.connection_protocol.clone()
    }

    #[wasm_bindgen(getter, js_name = "messageEndToEndMethod")]
    pub fn end_to_end_method(&self) -> Option<String> {
        self.end_to_end_method.clone()
    }
}
