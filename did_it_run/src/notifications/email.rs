use crate::config::Config;
use crate::notifications::{Dispatcher, NotificationInfo, NotifierError};
use crate::{DID_IT_RUN_EMAIL, DID_IT_RUN_NAME};
use common::types::Port;
use lettre::smtp::authentication::Mechanism;
use lettre::smtp::client::net::{ClientTlsParameters, NetworkStream};
use lettre::smtp::client::InnerClient;
use lettre::smtp::commands::{
    AuthCommand, EhloCommand, QuitCommand, StarttlsCommand,
};
use lettre::smtp::extension::ClientId;
use lettre::smtp::{ClientSecurity, SmtpClient, SmtpTransport};
use lettre::Transport;
use lettre_email::{self, EmailBuilder};
use native_tls::{self, Protocol, TlsConnector};
use serde_derive::Deserialize;
use std::convert::From;
use std::error;
use std::fmt;
use std::io;
use std::net::ToSocketAddrs;

const STARTTLS_PORT: Port = 587;
const AUTHENTICATION_MECHANISM: Mechanism = Mechanism::Plain;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct EmailConfig {
    pub recipients: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct SmtpCredentials {
    hostname: String,
    port: Option<Port>,
    username: String,
    password: String,
}

pub struct Mailer {
    mailer: SmtpTransport,
    config: EmailConfig,
}

#[derive(Debug)]
pub enum MailerError {
    Io(io::Error),
    LettreEmail(lettre_email::error::Error),
    MissingCredentials,
    NoEmailConfig,
    SmtpError(lettre::smtp::error::Error),
    Tls(native_tls::Error),
}

impl Mailer {
    pub fn new(
        config: Config,
        credentials: SmtpCredentials,
    ) -> Result<Self, MailerError> {
        let email_config = config.email.ok_or(MailerError::NoEmailConfig)?;
        let hostname = credentials.hostname;
        let port = credentials.port.unwrap_or(STARTTLS_PORT);
        let socket_addr = (hostname.as_str(), port)
            .to_socket_addrs()?
            .next()
            .ok_or(lettre::smtp::error::Error::Resolution)?;
        let mut tls_builder = TlsConnector::builder();
        tls_builder.min_protocol_version(Some(Protocol::Tlsv12));
        let tls_parameters =
            ClientTlsParameters::new(hostname, tls_builder.build()?);
        let client_security = ClientSecurity::Required(tls_parameters.clone());
        let smtp_credentials = lettre::smtp::authentication::Credentials::new(
            credentials.username,
            credentials.password,
        );
        if config.validate {
            let ehlo_command = EhloCommand::new(ClientId::hostname());
            let auth_command = AuthCommand::new(
                AUTHENTICATION_MECHANISM,
                smtp_credentials.clone(),
                None,
            )?;
            let mut client: InnerClient<NetworkStream> = InnerClient::new();
            client.connect(&socket_addr, config.timeout, None)?;
            client.command(&ehlo_command)?;
            client.command(StarttlsCommand)?;
            client.upgrade_tls_stream(&tls_parameters)?;
            client.command(&ehlo_command)?;
            client.command(&auth_command)?;
            client.command(QuitCommand)?;
        }
        let mailer = SmtpClient::new(socket_addr, client_security)?
            .credentials(smtp_credentials)
            .smtp_utf8(true)
            .authentication_mechanism(AUTHENTICATION_MECHANISM)
            .transport();
        Ok(Mailer {
            mailer,
            config: email_config,
        })
    }
}

impl Dispatcher for Mailer {
    fn dispatch_notification(
        &mut self,
        info: NotificationInfo,
    ) -> Result<(), NotifierError> {
        let subject = info.brief;
        let html_body = info.html_details;
        let plaintext_body = info.details;
        let mut builder = EmailBuilder::new()
            .from((DID_IT_RUN_EMAIL, DID_IT_RUN_NAME))
            .subject(subject)
            .alternative(html_body, plaintext_body);
        for recipient in &self.config.recipients {
            builder = builder.to(recipient.clone());
        }
        let email = builder.build().map_err(MailerError::LettreEmail)?.into();
        self.mailer.send(email).map_err(MailerError::SmtpError)?;
        Ok(())
    }
}

impl error::Error for MailerError {}

impl fmt::Display for MailerError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MailerError::Io(ref err) => write!(formatter, "IO error: {}", err),
            MailerError::LettreEmail(ref err) => err.fmt(formatter),
            MailerError::MissingCredentials => {
                write!(formatter, "No smtp credentials provided.")
            },
            MailerError::NoEmailConfig => {
                write!(formatter, "No email config provided.")
            },
            MailerError::SmtpError(ref err) => err.fmt(formatter),
            MailerError::Tls(ref err) => err.fmt(formatter),
        }
    }
}

impl From<io::Error> for MailerError {
    fn from(err: io::Error) -> Self {
        MailerError::Io(err)
    }
}

impl From<lettre_email::error::Error> for MailerError {
    fn from(err: lettre_email::error::Error) -> Self {
        match err {
            lettre_email::error::Error::Io(err) => MailerError::Io(err),
            _ => MailerError::LettreEmail(err),
        }
    }
}

impl From<lettre::smtp::error::Error> for MailerError {
    fn from(err: lettre::smtp::error::Error) -> Self {
        match err {
            lettre::smtp::error::Error::Io(err) => MailerError::Io(err),
            lettre::smtp::error::Error::Tls(err) => MailerError::Tls(err),
            _ => MailerError::SmtpError(err),
        }
    }
}

impl From<native_tls::Error> for MailerError {
    fn from(err: native_tls::Error) -> Self {
        MailerError::Tls(err)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::notifications::notification_info::test::NOTIFICATION_INFO;
    use crate::test::PROJECT_ROOT_PATH;
    use lazy_static::lazy_static;
    use mailin_embedded::{
        self, AuthMechanism, AuthResult, DataResult, Server, SslConfig,
    };
    use mailparse;
    use mailparse::MailHeaderMap;
    use matches::assert_matches;
    use std::io;
    use std::io::Write;
    use std::path::PathBuf;
    use std::result;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::sync::mpsc::Sender;

    const HOSTNAME: &str = "localhost";
    const USERNAME: &str = "username";
    const PASSWORD: &str = "password";

    lazy_static! {
        static ref TLS_FIXTURE_PATH: PathBuf =
            PROJECT_ROOT_PATH.join("tests/fixtures/tls/");
        static ref TLS_KEY_PATH: PathBuf = TLS_FIXTURE_PATH.join("test.key");
        static ref TLS_CERT_PATH: PathBuf = TLS_FIXTURE_PATH.join("test.crt");
        // Tests are run concurrently, therefore, each test's mail server must
        // run on a unique port.
        static ref PORT_INCREMENTER: AtomicUsize = AtomicUsize::new(9000);
    }

    // Workaround: SslConfig doesn't implement Clone so we can't make this a
    // lazy_static
    fn ssl_config() -> SslConfig {
        SslConfig::SelfSigned {
            cert_path: TLS_CERT_PATH.to_string_lossy().into(),
            key_path: TLS_KEY_PATH.to_string_lossy().into(),
        }
    }

    struct Writer {
        sender: Sender<Vec<u8>>,
    }

    #[derive(Clone)]
    struct Handler {
        auth_sender: Sender<bool>,
        data_sender: Sender<Vec<u8>>,
    }

    impl EmailConfig {
        pub fn new<I, S>(recipients: I) -> Self
        where
            I: IntoIterator<Item = S>,
            S: Into<String>,
        {
            EmailConfig {
                recipients: recipients
                    .into_iter()
                    .map(|recipient| recipient.into())
                    .collect(),
            }
        }
    }

    impl SmtpCredentials {
        pub fn new<S>(hostname: S, port: Port, username: S, password: S) -> Self
        where
            S: Into<String>,
        {
            SmtpCredentials {
                hostname: hostname.into(),
                port: Some(port),
                username: username.into(),
                password: password.into(),
            }
        }

        fn new_valid() -> Self {
            let port = PORT_INCREMENTER.fetch_add(1, Ordering::SeqCst) as Port;
            SmtpCredentials {
                hostname: HOSTNAME.to_string(),
                port: Some(port),
                username: USERNAME.to_string(),
                password: PASSWORD.to_string(),
            }
        }

        fn new_invalid() -> Self {
            let port = PORT_INCREMENTER.fetch_add(1, Ordering::SeqCst) as Port;
            let credentials = SmtpCredentials {
                hostname: HOSTNAME.to_string(),
                port: Some(port),
                username: "bad_username".to_string(),
                password: "bad_password".to_string(),
            };
            assert_ne!(credentials.username, USERNAME);
            assert_ne!(credentials.password, PASSWORD);
            credentials
        }
    }

    impl fmt::Debug for Mailer {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "config: {:?}", self.config)
        }
    }

    impl Write for Writer {
        fn write(&mut self, buffer: &[u8]) -> result::Result<usize, io::Error> {
            match self.sender.send(buffer.to_vec()) {
                Ok(()) => Ok(buffer.len()),
                Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        fn flush(&mut self) -> result::Result<(), io::Error> {
            Ok(())
        }
    }

    impl mailin_embedded::Handler for Handler {
        fn data(
            &mut self,
            _domain: &str,
            _from: &str,
            _is8bit: bool,
            _to: &[String],
        ) -> DataResult {
            let writer = Writer {
                sender: self.data_sender.clone(),
            };
            DataResult::Ok(Box::new(writer))
        }

        fn auth_plain(
            &mut self,
            _authorization_id: &str,
            authentication_id: &str,
            password: &str,
        ) -> AuthResult {
            if authentication_id == USERNAME && password == PASSWORD {
                self.auth_sender.send(true).unwrap();
                AuthResult::Ok
            } else {
                self.auth_sender.send(false).unwrap();
                AuthResult::InvalidCredentials
            }
        }
    }

    fn body_for_mimetype(
        email: &mailparse::ParsedMail,
        mimetype: &str,
    ) -> String {
        email
            .subparts
            .first()
            .unwrap()
            .subparts
            .iter()
            .filter(|part| part.ctype.mimetype == mimetype)
            .next()
            .unwrap()
            .get_body()
            .unwrap()
    }

    #[test]
    fn sends_mail_on_success() {
        let config = Config {
            email: Some(EmailConfig::new(vec![
                "recipient1@localhost",
                "recipient2@localhost",
                "recipient3@localhost",
            ])),
            ..Default::default()
        };
        let credentials = SmtpCredentials::new_valid();
        let server_address =
            format!("{}:{}", credentials.hostname, credentials.port.unwrap());
        let (auth_sender, auth_success_receiver) = mpsc::channel();
        let (data_sender, data_receiver) = mpsc::channel();
        let handler = Handler {
            auth_sender,
            data_sender,
        };
        let mut server = Server::new(handler);
        server
            .with_ssl(ssl_config())
            .with_num_threads(1)
            .with_auth(AuthMechanism::Plain)
            .with_addr(server_address)
            .unwrap();
        let handle = server.serve().unwrap();

        let mut mailer = Mailer::new(config.clone(), credentials).unwrap();
        assert!(auth_success_receiver.iter().next().unwrap());

        let result = mailer.dispatch_notification(NOTIFICATION_INFO.clone());
        assert!(auth_success_receiver.iter().next().unwrap());
        assert!(result.is_ok());

        handle.stop();
        assert_eq!(auth_success_receiver.iter().count(), 0);

        let email = data_receiver.iter().flatten().collect::<Vec<_>>();
        let email = mailparse::parse_mail(&email).unwrap();
        let headers = &email.headers;

        let subject = headers.get_first_value("Subject").unwrap().unwrap();
        assert_eq!(subject, NOTIFICATION_INFO.brief);

        let to = headers.get_first_value("To").unwrap().unwrap();
        for recipient in config.email.unwrap().recipients {
            assert!(to.contains(&recipient));
        }

        let from = headers.get_first_value("From").unwrap().unwrap();
        let expected_from =
            format!("\"{}\" <{}>", DID_IT_RUN_NAME, DID_IT_RUN_EMAIL);
        assert_eq!(from, expected_from);

        let plaintext_body = body_for_mimetype(&email, "text/plain");
        assert_eq!(plaintext_body.trim(), NOTIFICATION_INFO.details);

        let html_body = body_for_mimetype(&email, "text/html");
        assert_eq!(html_body.trim(), NOTIFICATION_INFO.html_details);
    }

    #[test]
    fn creates_mailer_with_bad_credentials_and_no_validation() {
        let config = Config {
            email: Some(EmailConfig::new(vec!["someone@localhost"])),
            validate: false,
            ..Default::default()
        };
        let credentials = SmtpCredentials::new_invalid();
        let server_address =
            format!("{}:{}", credentials.hostname, credentials.port.unwrap());
        let (auth_sender, auth_success_receiver) = mpsc::channel();
        let (data_sender, data_receiver) = mpsc::channel();
        let handler = Handler {
            auth_sender,
            data_sender,
        };
        let mut server = Server::new(handler);
        server
            .with_ssl(ssl_config())
            .with_num_threads(1)
            .with_auth(AuthMechanism::Plain)
            .with_addr(server_address)
            .unwrap();
        let handle = server.serve().unwrap();
        let mailer = Mailer::new(config.clone(), credentials);
        handle.stop();
        assert!(mailer.is_ok());
        assert_eq!(auth_success_receiver.iter().count(), 0);
        assert_eq!(data_receiver.iter().count(), 0);
    }

    #[test]
    fn fails_to_create_mailer_with_bad_credentials() {
        let config = Config {
            email: Some(EmailConfig::new(vec!["someone@localhost"])),
            ..Default::default()
        };
        let credentials = SmtpCredentials::new_invalid();
        let server_address =
            format!("{}:{}", credentials.hostname, credentials.port.unwrap());
        let (auth_sender, auth_success_receiver) = mpsc::channel();
        let (data_sender, data_receiver) = mpsc::channel();
        let handler = Handler {
            auth_sender,
            data_sender,
        };
        let mut server = Server::new(handler);
        server
            .with_ssl(ssl_config())
            .with_num_threads(1)
            .with_auth(AuthMechanism::Plain)
            .with_addr(server_address)
            .unwrap();
        let handle = server.serve().unwrap();
        let mailer = Mailer::new(config.clone(), credentials);
        handle.stop();
        assert!(mailer.is_err());
        assert_matches!(mailer.unwrap_err(), MailerError::SmtpError(_));
        assert!(!auth_success_receiver.iter().next().unwrap());
        assert_eq!(auth_success_receiver.iter().count(), 0);
        assert_eq!(data_receiver.iter().count(), 0);
    }

    #[test]
    fn fails_to_send_mail_with_bad_credentials() {
        let config = Config {
            email: Some(EmailConfig::new(vec!["someone@localhost"])),
            validate: false,
            ..Default::default()
        };
        let credentials = SmtpCredentials::new_invalid();
        let server_address =
            format!("{}:{}", credentials.hostname, credentials.port.unwrap());
        let (auth_sender, auth_success_receiver) = mpsc::channel();
        let (data_sender, data_receiver) = mpsc::channel();
        let handler = Handler {
            auth_sender,
            data_sender,
        };
        let mut server = Server::new(handler);
        server
            .with_ssl(ssl_config())
            .with_num_threads(1)
            .with_auth(AuthMechanism::Plain)
            .with_addr(server_address)
            .unwrap();
        let handle = server.serve().unwrap();

        let mut mailer = Mailer::new(config.clone(), credentials).unwrap();
        let result = mailer.dispatch_notification(NOTIFICATION_INFO.clone());
        handle.stop();

        assert!(!auth_success_receiver.iter().next().unwrap());
        assert_eq!(auth_success_receiver.iter().count(), 0);
        assert_eq!(data_receiver.iter().count(), 0);
        assert!(result.is_err());
        assert_matches!(
            result.unwrap_err(),
            NotifierError::Email(MailerError::SmtpError(_))
        );
    }

    #[test]
    fn error_conversion() {
        let message = "oh no!";

        let io_err = io::Error::new(io::ErrorKind::Other, message);
        let mailer_error = MailerError::from(io_err);
        assert_matches!(mailer_error, MailerError::Io(_));

        let io_err = io::Error::new(io::ErrorKind::Other, message);
        let lettre_email_err = lettre_email::error::Error::Io(io_err);
        let mailer_error = MailerError::from(lettre_email_err);
        assert_matches!(mailer_error, MailerError::Io(_));

        let lettre_email_err = lettre_email::error::Error::CannotParseFilename;
        let mailer_error = MailerError::from(lettre_email_err);
        assert_matches!(mailer_error, MailerError::LettreEmail(_));

        let io_err = io::Error::new(io::ErrorKind::Other, message);
        let lettre_smtp_err = lettre::smtp::error::Error::Io(io_err);
        let mailer_error = MailerError::from(lettre_smtp_err);
        assert_matches!(mailer_error, MailerError::Io(_));

        // Hack: Create a native_tls::Error by trying to make a bad certificte
        // We cannot expect_err because native_tls::Certificate does not
        // implement fmt::Debug
        let bad_certificate =
            native_tls::Certificate::from_pem("bad certificate".as_bytes());
        let tls_err = match bad_certificate {
            Ok(_) => panic!("Did not produce native_tls error."),
            Err(err) => err,
        };
        let lettre_smtp_err = lettre::smtp::error::Error::Tls(tls_err);
        let mailer_error = MailerError::from(lettre_smtp_err);
        assert_matches!(mailer_error, MailerError::Tls(_));

        let lettre_smtp_err = lettre::smtp::error::Error::Client(message);
        let mailer_error = MailerError::from(lettre_smtp_err);
        assert_matches!(mailer_error, MailerError::SmtpError(_));

        // Hack: Create a native_tls::Error by trying to make a bad certificte
        // We cannot expect_err because native_tls::Certificate does not
        // implement fmt::Debug
        let bad_certificate =
            native_tls::Certificate::from_pem("bad certificate".as_bytes());
        let tls_err = match bad_certificate {
            Ok(_) => panic!("Did not produce native_tls error."),
            Err(err) => err,
        };
        let mailer_error = MailerError::from(tls_err);
        assert_matches!(mailer_error, MailerError::Tls(_));
    }

    // Hacks to minimize coverage report errors
    #[test]
    fn maximize_coverage() {
        let io_err = io::Error::new(io::ErrorKind::Other, "oh no!");
        let mailer_err = MailerError::Io(io_err);
        let _ = format!("{:?}", mailer_err);

        let lettre_email_err = lettre_email::error::Error::CannotParseFilename;
        let mailer_err = MailerError::LettreEmail(lettre_email_err);
        let _ = format!("{:?}", mailer_err);

        let mailer_err = MailerError::NoEmailConfig;
        let _ = format!("{:?}", mailer_err);

        // Hack: Create a native_tls::Error by trying to make a bad certificte
        // We cannot expect_err because native_tls::Certificate does not
        // implement fmt::Debug
        let bad_certificate =
            native_tls::Certificate::from_pem("bad certificate".as_bytes());
        let tls_err = match bad_certificate {
            Ok(_) => panic!("Did not produce native_tls error."),
            Err(err) => err,
        };
        let mailer_err = MailerError::Tls(tls_err);
        let _ = format!("{:?}", mailer_err);

        let config = Config {
            email: Some(EmailConfig::new(vec!["someone@localhost"])),
            validate: false,
            ..Default::default()
        };
        let credentials = SmtpCredentials::new_valid();
        let mailer = Mailer::new(config, credentials).unwrap();
        assert_eq!(
            "config: EmailConfig { recipients: [\"someone@localhost\"] }",
            format!("{:?}", mailer)
        );

        let (sender, _) = mpsc::channel();
        let mut writer = Writer { sender };
        assert!(writer.write(&[]).is_err());
        assert!(writer.flush().is_ok());
    }
}
