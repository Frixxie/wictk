use wictk_core::{MetAlert, Severity};

use crate::notifications::{Notification, Priority, Tag};

impl From<Severity> for Priority {
    fn from(value: Severity) -> Self {
        match value {
            Severity::Yellow => Priority::Default,
            Severity::Orange => Priority::High,
            Severity::Red => Priority::Urgent,
        }
    }
}

impl From<MetAlert> for Notification {
    fn from(value: MetAlert) -> Self {
        let tag: Tag = match value.event.as_str() {
            "gale" => Tag::Wind,
            _ => Tag::Warning,
        };
        Notification::new(
            value.title,
            From::from(value.severity),
            tag,
            value.description,
        )
    }
}
