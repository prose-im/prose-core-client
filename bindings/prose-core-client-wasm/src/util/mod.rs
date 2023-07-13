use prose_xmpp::TimeProvider;
use std::time::SystemTime;

#[derive(Default)]
pub struct WasmTimeProvider {}

impl TimeProvider for WasmTimeProvider {
    fn now(&self) -> SystemTime {
        use js_sys::Date;
        use std::time::Duration;

        let now_ms = Date::now();
        let secs = (now_ms / 1000.0).floor() as u64;
        let nanos = (now_ms % 1000.0 * 1_000_000.0) as u32; // Convert remaining milliseconds to nanoseconds

        let duration = Duration::new(secs, nanos);
        SystemTime::UNIX_EPOCH + duration
    }
}
