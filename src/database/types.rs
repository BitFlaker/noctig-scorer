use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct RecentProject {
    pub name: String,
    pub path: String,
    pub last_opened: DateTime<Utc>
}
