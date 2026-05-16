/// Returns the current Unix timestamp in seconds.
///
/// On native targets: uses `std::time::SystemTime`.
/// On wasm32 targets: uses `js_sys::Date::now()` (browser clock).
pub fn unix_now() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
}
