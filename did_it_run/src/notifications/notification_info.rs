use crate::duration_format::duration_format;
use crate::exit_code;
use crate::notifications::Event;

#[derive(Clone)]
pub struct NotificationInfo {
    pub brief: String,
    pub details: String,
    pub html_details: String,
}

impl From<Event> for NotificationInfo {
    fn from(event: Event) -> Self {
        match event {
            Event::Finished {
                incantation,
                exit_code,
                elapsed_time,
            } => {
                let command = incantation.command.to_string_lossy();
                let brief = if exit_code == exit_code::SUCCESS {
                    format!("`{}` succeeded", command)
                } else {
                    format!("`{}` failed", command)
                };
                let details = if exit_code == exit_code::SUCCESS {
                    format!(
                        "`{}` succeeded in {}.",
                        incantation,
                        duration_format(&elapsed_time)
                    )
                } else {
                    format!(
                        "`{}` failed with exit code {} in {}.",
                        incantation,
                        exit_code,
                        duration_format(&elapsed_time)
                    )
                };
                let html_details = if exit_code == exit_code::SUCCESS {
                    format!(
                        "<code>{}</code> succeeded in {}.",
                        incantation,
                        duration_format(&elapsed_time)
                    )
                } else {
                    format!(
                        "<code>{}</code> failed with exit code {} in {}.",
                        incantation,
                        exit_code,
                        duration_format(&elapsed_time)
                    )
                };
                NotificationInfo {
                    brief,
                    details,
                    html_details,
                }
            },
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::exit_code::{FAILURE, SUCCESS};
    use crate::incantation::Incantation;
    use lazy_static::lazy_static;
    use std::time::Duration;

    lazy_static! {
        pub static ref NOTIFICATION_INFO: NotificationInfo = NotificationInfo {
            brief: "Notification summary".to_string(),
            details: "Notification details".to_string(),
            html_details: "<p>Notification details</p>".to_string(),
        };
    }

    #[test]
    fn creates_info_from_event_finish_success() {
        let event = Event::Finished {
            incantation: Incantation::new("foo", vec!["bar", "baz"]),
            exit_code: SUCCESS,
            elapsed_time: Duration::from_secs(2),
        };
        let info: NotificationInfo = event.into();
        assert_eq!(info.brief, "`foo` succeeded");
        assert_eq!(info.details, "`foo bar baz` succeeded in 2s.");
        assert_eq!(
            info.html_details,
            "<code>foo bar baz</code> succeeded in 2s."
        );
    }

    #[test]
    fn creates_info_from_event_finish_failure() {
        let exit_code = FAILURE;
        let event = Event::Finished {
            incantation: Incantation::new("foo", vec!["bar", "baz"]),
            exit_code,
            elapsed_time: Duration::from_secs(2),
        };
        let info: NotificationInfo = event.into();
        assert_eq!(info.brief, "`foo` failed");
        let failure_message =
            format!("`foo bar baz` failed with exit code {} in 2s.", exit_code);
        assert_eq!(info.details, failure_message);
        let failure_message = format!(
            "<code>foo bar baz</code> failed with exit code {} in 2s.",
            exit_code
        );
        assert_eq!(info.html_details, failure_message);
    }
}
