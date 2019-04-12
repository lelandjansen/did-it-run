pub mod desktop;
pub mod email;
mod notification_info;

use crate::config::{Config, Credentials};
use crate::exit_code::ExitCode;
use crate::incantation::Incantation;
use crate::notifications::desktop::{DesktopError, DesktopNotifier};
use crate::notifications::email::{Mailer, MailerError};
use crate::notifications::notification_info::NotificationInfo;
use std::error;
use std::fmt;
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum Event {
    Finished {
        incantation: Incantation,
        exit_code: ExitCode,
        elapsed_time: Duration,
    },
}

pub struct Notifier {
    dispatchers: Vec<Box<Dispatcher>>,
}

#[derive(Debug)]
pub enum NotifierError {
    Desktop(DesktopError),
    Email(MailerError),
}

trait Dispatcher {
    fn dispatch_notification(
        &mut self,
        info: NotificationInfo,
    ) -> Result<(), NotifierError>;
}

impl Notifier {
    pub fn new(
        config: Config,
        credentials: Credentials,
    ) -> Result<Self, NotifierError> {
        let mut dispatchers: Vec<Box<Dispatcher>> = vec![];
        if config.desktop_notifications {
            let desktop_notifier = DesktopNotifier::new()?;
            dispatchers.push(Box::new(desktop_notifier));
        }

        if config.email.is_some() && credentials.smtp.is_none() {
            return Err(MailerError::MissingCredentials.into());
        }
        if let (Some(_), Some(credentials)) = (&config.email, credentials.smtp)
        {
            let mailer = Mailer::new(config, credentials)?;
            dispatchers.push(Box::new(mailer));
        }

        Ok(Notifier { dispatchers })
    }

    pub fn notify(&mut self, event: Event) -> Result<(), NotifierError> {
        let info = NotificationInfo::from(event);
        for dispatcher in &mut self.dispatchers {
            dispatcher.dispatch_notification(info.clone())?;
        }
        Ok(())
    }
}

impl error::Error for NotifierError {}

impl fmt::Display for NotifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NotifierError::Desktop(ref err) => err.fmt(formatter),
            NotifierError::Email(ref err) => err.fmt(formatter),
        }
    }
}

impl From<DesktopError> for NotifierError {
    fn from(err: DesktopError) -> Self {
        NotifierError::Desktop(err)
    }
}

impl From<MailerError> for NotifierError {
    fn from(err: MailerError) -> Self {
        NotifierError::Email(err)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::LATEST_CONFIG_VERSION;
    use crate::exit_code::SUCCESS;
    use crate::notifications::email::{EmailConfig, SmtpCredentials};
    use lazy_static::lazy_static;
    use matches::assert_matches;

    lazy_static! {
        pub static ref EVENT_FINISHED: Event = Event::Finished {
            incantation: Incantation::new("foo", vec!["bar", "baz"]),
            exit_code: SUCCESS,
            elapsed_time: Duration::from_secs(2),
        };
    }

    impl fmt::Debug for Notifier {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "dispatchers: {}", self.dispatchers.len())
        }
    }

    #[test]
    fn dispatches_notifications() {
        let config = Config::default();
        let credentials = Credentials {
            version: LATEST_CONFIG_VERSION.clone(),
            smtp: None,
        };
        let mut notifier = Notifier::new(config, credentials).unwrap();
        let result = notifier.notify(EVENT_FINISHED.clone());
        assert!(result.is_ok());
    }

    #[test]
    fn adds_mailer_to_dispatchers() {
        let config = Config {
            email: Some(EmailConfig {
                recipients: vec!["someone@example.com".into()],
            }),
            validate: false,
            ..Default::default()
        };
        let credentials = Credentials {
            version: LATEST_CONFIG_VERSION.clone(),
            smtp: Some(SmtpCredentials::new(
                "example.com",
                1234,
                "username",
                "password",
            )),
        };
        let notifier = Notifier::new(config, credentials);
        assert!(notifier.is_ok());
    }

    #[test]
    fn returns_error_with_missing_credentials() {
        let config = Config {
            desktop_notifications: false,
            email: Some(EmailConfig::new(vec![
                "someone@something.com".to_string()
            ])),
            ..Default::default()
        };
        let credentials = Credentials {
            version: LATEST_CONFIG_VERSION.clone(),
            smtp: None,
        };
        let notifier = Notifier::new(config, credentials);
        assert!(notifier.is_err());
        assert_matches!(
            notifier.unwrap_err(),
            NotifierError::Email(MailerError::MissingCredentials)
        );
    }

    // Hacks to minimize coverage report errors
    #[test]
    #[cfg(target_os = "linux")]
    fn maximize_coverage_report() {
        let error = NotifierError::Desktop(DesktopError::AlreadyInitialized);
        let _ = format!("{:?}", error);
        let error = NotifierError::Email(MailerError::MissingCredentials);
        let _ = format!("{:?}", error);

        let credentials = Credentials {
            version: LATEST_CONFIG_VERSION.clone(),
            smtp: None,
        };
        let notifier = Notifier::new(Config::default(), credentials).unwrap();
        let _ = format!("{:?}", notifier);

        let _: NotifierError = DesktopError::AlreadyInitialized.into();
    }
}
