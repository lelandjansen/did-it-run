use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

pub type Emails = BTreeSet<Email>;

#[derive(Debug, Deserialize, Eq, Serialize)]
pub struct Email {
    pub email: String,
    pub verified: bool,
    pub source: EmailSource,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EmailSource {
    User,
    GitHub,
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.email.to_lowercase() == other.email.to_lowercase() &&
            self.verified == other.verified
    }
}

impl Ord for Email {
    fn cmp(&self, other: &Self) -> Ordering {
        self.verified
            .cmp(&other.verified)
            .then(self.email.to_lowercase().cmp(&other.email.to_lowercase()))
    }
}

impl PartialOrd for Email {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for EmailSource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn email_eq() {
        let a_email = "a@x.com".to_string();
        let b_email = "b@x.com".to_string();
        let a1 = Email {
            email: a_email.clone(),
            verified: false,
            source: EmailSource::User,
        };
        let a2 = Email {
            email: a_email.clone(),
            verified: false,
            source: EmailSource::User,
        };
        assert_eq!(a1, a2);

        let lowercase = Email {
            email: a_email.to_lowercase(),
            verified: false,
            source: EmailSource::User,
        };
        let uppercase = Email {
            email: a_email.to_uppercase(),
            verified: false,
            source: EmailSource::User,
        };
        assert_eq!(lowercase, uppercase);

        let a = Email {
            email: a_email.clone(),
            verified: false,
            source: EmailSource::User,
        };
        let b = Email {
            email: b_email,
            verified: false,
            source: EmailSource::User,
        };
        assert_ne!(a, b);

        let a1 = Email {
            email: a_email.clone(),
            verified: false,
            source: EmailSource::User,
        };
        let a2 = Email {
            email: a_email,
            verified: true,
            source: EmailSource::User,
        };
        assert_ne!(a1, a2);
    }

    #[test]
    fn email_cmp() {
        let a_email = "a@x.com".to_string();
        let b_email = "b@x.com".to_string();
        let not_verified = Email {
            email: b_email.clone(),
            verified: false,
            source: EmailSource::User,
        };
        let verified = Email {
            email: a_email.clone(),
            verified: true,
            source: EmailSource::User,
        };
        assert!(not_verified < verified);

        let a = Email {
            email: a_email,
            verified: false,
            source: EmailSource::User,
        };
        let b = Email {
            email: b_email,
            verified: false,
            source: EmailSource::User,
        };
        assert!(a < b);
    }
}
