use crate::notifications::email::{EmailConfig, SmtpCredentials};
use lazy_static::lazy_static;
use semver::{SemVerError, Version};
use serde::de::DeserializeOwned;
use serde_derive::Deserialize;
use std::default::Default;
use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use toml;

pub type TimeoutInput = u64;

lazy_static! {
    static ref HOME_DIR: PathBuf = dirs::home_dir().unwrap_or_default();
    pub static ref LATEST_CONFIG_VERSION: Version =
        Version::parse("0.0.1").unwrap();
    pub static ref LATEST_CREDENTIALS_VERSION: Version =
        Version::parse("0.0.1").unwrap();
    static ref DEFAULT_DIRECTORIES: Vec<PathBuf> =
        vec![HOME_DIR.join("diditrun"), HOME_DIR.join(".diditrun")];
    pub static ref DEFAULT_CONFIG_FILES: Vec<PathBuf> = DEFAULT_DIRECTORIES
        .iter()
        .map(|dir| dir.join("config.toml"))
        .collect();
    pub static ref DEFAULT_CREDENTIALS_FILES: Vec<PathBuf> =
        DEFAULT_DIRECTORIES
            .iter()
            .map(|dir| dir.join("credentials.toml"))
            .collect();
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct UserConfig {
    pub version: Option<String>,
    pub desktop_notifications: Option<bool>,
    pub email: Option<EmailConfig>,
    pub validate: Option<bool>,
    pub timeout: Option<TimeoutInput>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub version: Version,
    pub desktop_notifications: bool,
    pub email: Option<EmailConfig>,
    pub validate: bool,
    pub timeout: Option<Duration>,
}

#[derive(Debug, Default)]
pub struct MergeOptions {
    pub no_email: bool,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub struct UserCredentials {
    pub version: Option<String>,
    pub smtp: Option<SmtpCredentials>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Credentials {
    pub version: Version,
    pub smtp: Option<SmtpCredentials>,
}

#[derive(Debug)]
pub enum LoadConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
}

#[derive(Debug)]
pub enum ConfigError {
    MalformedVersion(SemVerError),
    InvalidVersion(Version),
}

pub fn load_file<T: Default + DeserializeOwned>(
    file: Option<PathBuf>,
    default: Vec<PathBuf>,
) -> Result<T, LoadConfigError> {
    let file = file.or_else(|| {
        default
            .iter()
            .filter(|path| path.exists())
            .collect::<Vec<_>>()
            .first()
            .map(|path| path.to_owned().to_owned())
    });
    let config = match file {
        Some(file) => {
            let text = fs::read_to_string(&file)?;
            toml::from_str(&text)?
        },
        None => Default::default(),
    };
    Ok(config)
}

pub fn merge(
    cli_config: UserConfig,
    file_config: UserConfig,
    options: MergeOptions,
) -> UserConfig {
    let email = if options.no_email {
        None
    } else {
        cli_config.email.or(file_config.email)
    };
    let desktop_notifications = cli_config
        .desktop_notifications
        .or(file_config.desktop_notifications);
    UserConfig {
        version: cli_config.version.or(file_config.version),
        desktop_notifications,
        email,
        validate: cli_config.validate.or(file_config.validate),
        timeout: cli_config.timeout.or(file_config.timeout),
    }
}

impl Config {
    pub fn from_user_config(
        user_config: UserConfig,
    ) -> Result<Self, ConfigError> {
        let version = if let Some(version) = user_config.version {
            Version::parse(&version)?
        } else {
            LATEST_CONFIG_VERSION.clone()
        };
        let email = user_config.email.and_then(|email| {
            if email.recipients.is_empty() {
                None
            } else {
                Some(EmailConfig {
                    recipients: email.recipients,
                })
            }
        });
        let desktop_notifications =
            user_config.desktop_notifications.unwrap_or(true);
        if *LATEST_CREDENTIALS_VERSION < version {
            Err(ConfigError::InvalidVersion(version))
        } else {
            Ok(Config {
                version,
                desktop_notifications,
                email,
                validate: user_config.validate.unwrap_or(true),
                timeout: user_config.timeout.map(Duration::from_secs),
            })
        }
    }
}

impl Credentials {
    pub fn from_user_credentials(
        user_credentials: UserCredentials,
    ) -> Result<Self, ConfigError> {
        let version = if let Some(version) = user_credentials.version {
            Version::parse(&version)?
        } else {
            LATEST_CREDENTIALS_VERSION.clone()
        };
        if *LATEST_CREDENTIALS_VERSION < version {
            Err(ConfigError::InvalidVersion(version))
        } else {
            Ok(Credentials {
                version,
                smtp: user_credentials.smtp,
            })
        }
    }
}

impl fmt::Display for LoadConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LoadConfigError::Io(ref err) => err.fmt(formatter),
            LoadConfigError::Toml(ref err) => err.fmt(formatter),
        }
    }
}

impl error::Error for LoadConfigError {}

impl From<io::Error> for LoadConfigError {
    fn from(err: io::Error) -> Self {
        LoadConfigError::Io(err)
    }
}

impl From<toml::de::Error> for LoadConfigError {
    fn from(err: toml::de::Error) -> Self {
        LoadConfigError::Toml(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::MalformedVersion(ref err) => err.fmt(formatter),
            ConfigError::InvalidVersion(ref version) => write!(
                formatter,
                "Specified config file version ({}) is greater than the \
                 current version ({}).",
                version, *LATEST_CREDENTIALS_VERSION
            ),
        }
    }
}

impl error::Error for ConfigError {}

impl From<SemVerError> for ConfigError {
    fn from(err: SemVerError) -> Self {
        ConfigError::MalformedVersion(err)
    }
}

impl From<Version> for ConfigError {
    fn from(version: Version) -> Self {
        ConfigError::InvalidVersion(version)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::PROJECT_ROOT_PATH;
    use matches::assert_matches;

    lazy_static! {
        static ref CONFIG_FIXTURE_PATH: PathBuf =
            PROJECT_ROOT_PATH.join("tests/fixtures/diditrun/");
        static ref DEFAULT_CONFIG_FIXTURE_PATH: Vec<PathBuf> =
            vec![CONFIG_FIXTURE_PATH.join("default-config.toml")];
        static ref DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH: Vec<PathBuf> =
            vec![CONFIG_FIXTURE_PATH.join("default-credentials.toml")];
    }

    impl Default for Config {
        fn default() -> Self {
            Config {
                version: LATEST_CONFIG_VERSION.clone(),
                desktop_notifications: true,
                email: None,
                validate: true,
                timeout: None,
            }
        }
    }

    #[test]
    fn loads_user_config_from_specified_file() {
        let config_file_path = CONFIG_FIXTURE_PATH.join("config.toml");
        let user_config = load_file::<UserConfig>(
            Some(config_file_path),
            DEFAULT_CONFIG_FIXTURE_PATH.to_vec(),
        );
        assert!(user_config.is_ok());
        let expected_user_config = UserConfig {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            desktop_notifications: Some(true),
            email: Some(EmailConfig {
                recipients: vec!["someone@example.com".to_string()],
            }),
            validate: Some(true),
            timeout: Some(42),
        };
        assert_eq!(user_config.unwrap(), expected_user_config);
    }

    #[test]
    fn loads_default_user_config_file_with_no_specified_file() {
        let user_config =
            load_file::<UserConfig>(None, DEFAULT_CONFIG_FIXTURE_PATH.to_vec());
        assert!(user_config.is_ok());
        let expected_user_config = UserConfig {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            desktop_notifications: Some(false),
            email: Some(EmailConfig {
                recipients: vec!["default@example.com".to_string()],
            }),
            validate: Some(false),
            timeout: Some(30),
        };
        assert_eq!(user_config.unwrap(), expected_user_config);
    }

    #[test]
    fn returns_default_user_config_with_no_specified_file_or_default_file() {
        let user_config = load_file::<UserConfig>(None, vec![]);
        assert!(user_config.is_ok());
        assert_eq!(user_config.unwrap(), UserConfig::default());
    }

    #[test]
    fn fails_to_load_user_config_from_malformed_file() {
        let config_file_path =
            CONFIG_FIXTURE_PATH.join("malformed-config.toml");
        let user_config = load_file::<UserConfig>(
            Some(config_file_path),
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_config.is_err());
        assert_matches!(user_config.unwrap_err(), LoadConfigError::Toml(_));
    }

    #[test]
    fn returns_error_if_user_config_file_does_not_exist() {
        let config_file_path = PathBuf::from("does/not/exist.toml");
        let user_config = load_file::<UserConfig>(
            Some(config_file_path),
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_config.is_err());
        assert_matches!(user_config.unwrap_err(), LoadConfigError::Io(_));
    }

    #[test]
    fn loads_user_credentials_from_file() {
        let credentials_file_path =
            CONFIG_FIXTURE_PATH.join("credentials.toml");
        let user_credentials = load_file::<UserCredentials>(
            Some(credentials_file_path),
            DEFAULT_CONFIG_FIXTURE_PATH.to_vec(),
        );
        assert!(user_credentials.is_ok());
        let expected_user_credentials = UserCredentials {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            smtp: Some(SmtpCredentials::new(
                "hostname", 587, "username", "password",
            )),
        };
        assert_eq!(user_credentials.unwrap(), expected_user_credentials);
    }

    #[test]
    fn loads_default_user_credentials_with_no_specified_file() {
        let user_credentials = load_file::<UserCredentials>(
            None,
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_credentials.is_ok());
        let expected_user_credentials = UserCredentials {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            smtp: Some(SmtpCredentials::new(
                "default_hostname",
                8587,
                "username",
                "password",
            )),
        };
        assert_eq!(user_credentials.unwrap(), expected_user_credentials);
    }

    #[test]
    fn fails_to_load_user_credentials_from_malformed_file() {
        let credentials_file_path =
            CONFIG_FIXTURE_PATH.join("malformed-credentials.toml");
        let user_credentials = load_file::<UserCredentials>(
            Some(credentials_file_path),
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_credentials.is_err());
        assert_matches!(
            user_credentials.unwrap_err(),
            LoadConfigError::Toml(_)
        );
    }

    #[test]
    fn returns_error_if_credentials_field_is_missing() {
        let credentials_file_path =
            CONFIG_FIXTURE_PATH.join("missing-credentials.toml");
        let user_credentials = load_file::<UserCredentials>(
            Some(credentials_file_path),
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_credentials.is_err());
        assert_matches!(
            user_credentials.unwrap_err(),
            LoadConfigError::Toml(_)
        );
    }

    #[test]
    fn returns_error_if_user_credentials_file_does_not_exist() {
        let credentials_file_path = PathBuf::from("does/not/exist.toml");
        let user_credentials = load_file::<UserCredentials>(
            Some(credentials_file_path),
            DEFAULT_CREDENTIALS_FILES_FIXTURE_PATH.to_vec(),
        );
        assert!(user_credentials.is_err());
        assert_matches!(user_credentials.unwrap_err(), LoadConfigError::Io(_));
    }

    #[test]
    fn merges_user_configs() {
        let cli_config = UserConfig {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            desktop_notifications: Some(true),
            email: Some(EmailConfig {
                recipients: vec!["cli_config@example.com".to_string()],
            }),
            validate: Some(true),
            timeout: Some(10),
        };
        let file_config: UserConfig = Default::default();
        let merged =
            merge(cli_config.clone(), file_config, MergeOptions::default());
        assert_eq!(merged, cli_config);

        let file_config = UserConfig {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            desktop_notifications: Some(false),
            email: Some(EmailConfig {
                recipients: vec!["file_config@example.com".to_string()],
            }),
            validate: Some(false),
            timeout: Some(30),
        };
        let merged = merge(
            cli_config.clone(),
            file_config.clone(),
            MergeOptions::default(),
        );
        assert_eq!(merged, cli_config);

        let cli_config = UserConfig {
            version: Some(LATEST_CONFIG_VERSION.to_string()),
            desktop_notifications: Some(true),
            email: None,
            validate: Some(true),
            timeout: None,
        };
        let expected = UserConfig {
            version: cli_config.version.clone(),
            desktop_notifications: cli_config.desktop_notifications,
            email: file_config.email.clone(),
            validate: cli_config.validate,
            timeout: file_config.timeout,
        };
        let merged = merge(cli_config, file_config, MergeOptions::default());
        assert_eq!(merged, expected);
    }

    #[test]
    fn creates_config_from_user_config() {
        let version = LATEST_CONFIG_VERSION.clone();
        let email_config = EmailConfig {
            recipients: vec!["someone@example.com".to_string()],
        };
        let desktop_notifications = true;
        let validate = true;
        let timeout = 12;
        let user_config = UserConfig {
            version: Some(version.to_string()),
            desktop_notifications: Some(desktop_notifications),
            email: Some(email_config.clone()),
            validate: Some(validate),
            timeout: Some(timeout),
        };
        let config = Config::from_user_config(user_config).unwrap();
        let expected_config = Config {
            version,
            desktop_notifications,
            email: Some(email_config),
            validate,
            timeout: Some(Duration::from_secs(timeout)),
        };
        assert_eq!(config, expected_config);
    }

    #[test]
    fn creates_config_from_user_config_with_latest_version_if_not_specified() {
        let user_config = UserConfig {
            version: None,
            ..Default::default()
        };
        let config = Config::from_user_config(user_config);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().version, *LATEST_CONFIG_VERSION);
    }

    #[test]
    fn config_substitutes_empty_email_list_with_none() {
        let user_config = UserConfig {
            email: Some(EmailConfig { recipients: vec![] }),
            ..Default::default()
        };
        let config = Config::from_user_config(user_config);
        assert!(config.is_ok());
        assert_eq!(config.unwrap().email, None);
    }

    #[test]
    fn creates_credentials_from_user_credentials() {
        let version = LATEST_CONFIG_VERSION.clone();
        let smtp_credentials =
            SmtpCredentials::new("hostname", 1234, "username", "password");
        let user_credentials = UserCredentials {
            version: Some(version.to_string()),
            smtp: Some(smtp_credentials.clone()),
        };
        let credentials =
            Credentials::from_user_credentials(user_credentials).unwrap();
        let expected_credentials = Credentials {
            version,
            smtp: Some(smtp_credentials),
        };
        assert_eq!(credentials, expected_credentials);
    }

    #[test]
    fn creates_credentials_from_user_credentials_no_version() {
        let smtp_credentials =
            SmtpCredentials::new("hostname", 1234, "username", "password");
        let user_credentials = UserCredentials {
            version: None,
            smtp: Some(smtp_credentials.clone()),
        };
        let credentials =
            Credentials::from_user_credentials(user_credentials).unwrap();
        let expected_credentials = Credentials {
            version: LATEST_CONFIG_VERSION.clone(),
            smtp: Some(smtp_credentials),
        };
        assert_eq!(credentials, expected_credentials);
    }

    #[test]
    fn returns_error_if_specified_version_is_greater_than_current_version() {
        let version = "123.45.67";
        let user_config = UserConfig {
            version: Some(version.to_string()),
            ..Default::default()
        };
        let config = Config::from_user_config(user_config);
        assert!(config.is_err());
        assert_matches!(config.unwrap_err(), ConfigError::InvalidVersion(_));
        let user_credentials = UserCredentials {
            version: Some(version.to_string()),
            ..Default::default()
        };
        let credentials = Credentials::from_user_credentials(user_credentials);
        assert!(credentials.is_err());
        assert_matches!(
            credentials.unwrap_err(),
            ConfigError::InvalidVersion(_)
        );
    }

    #[test]
    fn no_email_option_overrides_file_config_email() {
        let cli_config = UserConfig::default();
        let file_config = UserConfig {
            email: Some(EmailConfig {
                recipients: vec!["someone@example.com".to_string()],
            }),
            ..Default::default()
        };
        assert!(file_config.email.is_some());
        let merge_options = MergeOptions { no_email: true };
        let merged = merge(cli_config, file_config, merge_options);
        assert!(merged.email.is_none());
    }

    #[test]
    fn error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::Other, "oh no!");
        let load_config_err = LoadConfigError::from(io_err);
        assert_matches!(load_config_err, LoadConfigError::Io(_));

        let toml_err = toml::from_str::<UserConfig>("bad toml").unwrap_err();
        let load_config_err = LoadConfigError::from(toml_err);
        assert_matches!(load_config_err, LoadConfigError::Toml(_));

        let semver_err = Version::parse("bad version").unwrap_err();
        let config_err = ConfigError::from(semver_err);
        assert_matches!(config_err, ConfigError::MalformedVersion(_));

        let version = Version::parse("1.2.3").unwrap();
        let version_err = ConfigError::from(version);
        assert_matches!(version_err, ConfigError::InvalidVersion(_));
    }

    // Hacks to minimize coverage report errors
    #[test]
    fn maximize_coverage() {
        let io_err = io::Error::new(io::ErrorKind::Other, "oh no!");
        let load_config_err = LoadConfigError::Io(io_err);
        let _ = format!("{:?}", load_config_err);

        let toml_err = toml::from_str::<UserConfig>("bad toml").unwrap_err();
        let load_config_err = LoadConfigError::Toml(toml_err);
        let _ = format!("{:?}", load_config_err);

        let semver_err = Version::parse("bad version").unwrap_err();
        let config_err = ConfigError::MalformedVersion(semver_err);
        let _ = format!("{:?}", config_err);

        let version = Version::parse("1.2.3").unwrap();
        let config_err = ConfigError::InvalidVersion(version);
        let _ = format!("{:?}", config_err);
    }
}
