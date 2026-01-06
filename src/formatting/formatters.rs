use chrono::{DateTime, TimeDelta, Utc};
use std::sync::LazyLock;

pub static TIME_FORMATTERS: LazyLock<Vec<fn(u64, u64, u64) -> String>> = LazyLock::new(|| vec![
    format_offset_seconds,
    format_offset_time_string,
    format_time_string,
    format_datetime_string
]);

pub fn format_offset_seconds(_: u64, from: u64, to: u64) -> String {
    format!("{}s - {}s", from, to)
}

pub fn format_offset_time_string(_: u64, from: u64, to: u64) -> String {
    format!("{} - {}", hms_u64(from), hms_u64(to))
}

pub fn format_time_string(start_time: u64, from: u64, to: u64) -> String {
    let begin_time = DateTime::<Utc>::from_timestamp_secs(start_time as i64).unwrap();
    let time_from = begin_time.checked_add_signed(TimeDelta::seconds(from as i64)).unwrap();
    let time_to = begin_time.checked_add_signed(TimeDelta::seconds(to as i64)).unwrap();
    let from_str = time_from.format("%H:%M:%S").to_string();
    let to_str = time_to .format("%H:%M:%S").to_string();

    format!("{} - {}", from_str, to_str)
}

pub fn format_datetime_string(start_time: u64, from: u64, to: u64) -> String {
    let begin_time = DateTime::<Utc>::from_timestamp_secs(start_time as i64).unwrap();
    let time_from = begin_time.checked_add_signed(TimeDelta::seconds(from as i64)).unwrap();
    let time_to = begin_time.checked_add_signed(TimeDelta::seconds(to as i64)).unwrap();
    let from_str = time_from.format("%d-%m-%Y %H:%M:%S").to_string();
    let to_str = time_to .format("%d-%m-%Y %H:%M:%S").to_string();

    format!("{} - {}", from_str, to_str)
}

pub fn date_time_string(timestamp: u64) -> String {
    let time = DateTime::<Utc>::from_timestamp_secs(timestamp as i64).unwrap();
    time.format("%d-%m-%Y %H:%M:%S").to_string()
}

fn hms_u64(total_seconds: u64) -> String {
    let hours = total_seconds / 3600;
    let rem = total_seconds % 3600;
    let minutes = rem / 60;
    let seconds = rem % 60;
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
    else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

pub fn hms_separate(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    let mut parts = Vec::new();
    
    if hours > 0 {
        parts.push(format!("{}h", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}m", minutes));
    }
    if secs > 0 {
        parts.push(format!("{}s", secs));
    }
    
    if parts.is_empty() {
        "0s".to_string()
    } else {
        parts.join(" ")
    }
}