use fancy_duration::FancyDuration;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use wasm_bindgen::UnwrapThrowExt;
use web_time::{Duration, SystemTime, UNIX_EPOCH};

pub fn format_time(unix_time: i64) -> String {
    let time = OffsetDateTime::from_unix_timestamp(unix_time).unwrap_throw();
    let timestamp = time.format(&Rfc3339).unwrap_throw();
    let date = timestamp.split('T').next().unwrap_throw().to_string();

    let time = UNIX_EPOCH
        .checked_add(Duration::from_secs(unix_time as u64))
        .unwrap_throw();

    let duration = if let Ok(duration) = time.duration_since(SystemTime::now()) {
        let duration = FancyDuration::new(duration).to_string();
        duration.split(' ').next().unwrap_throw().to_string()
    } else {
        "Expired".to_string()
    };

    format!("{} ({})", date, duration)
}
