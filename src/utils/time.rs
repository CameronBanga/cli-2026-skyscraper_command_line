use chrono::{DateTime, Utc};

pub fn relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    let seconds = duration.num_seconds();
    if seconds < 0 {
        return "now".to_string();
    }

    if seconds < 60 {
        return format!("{}s", seconds);
    }

    let minutes = duration.num_minutes();
    if minutes < 60 {
        return format!("{}m", minutes);
    }

    let hours = duration.num_hours();
    if hours < 24 {
        return format!("{}h", hours);
    }

    let days = duration.num_days();
    if days < 30 {
        return format!("{}d", days);
    }

    if days < 365 {
        return format!("{}mo", days / 30);
    }

    format!("{}y", days / 365)
}
