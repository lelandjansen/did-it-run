use crate::config::{MergeOptions, TimeoutInput, UserConfig};
use crate::email::EmailConfig;
use crate::incantation::Incantation;
use crate::DID_IT_RUN_NAME;
use clap;
use clap::{crate_authors, crate_description, crate_version, Arg};
use std::default::Default;
use std::ffi::OsString;
use std::path::PathBuf;

const ARGUMENTS: &str = "ARGUMENTS";
const CONFIG_FILE: &str = "CONFIG_FILE";
const CREDENTIALS_FILE: &str = "CREDENTIALS_FILE";
const COMMAND: &str = "COMMAND";
const EMAIL: &str = "EMAIL";
const NO_EMAIL: &str = "NO_EMAIL";
const NO_VALIDATE: &str = "NO_VALIDATE";
const TIMEOUT: &str = "TIMEOUT";

#[derive(Debug)]
pub struct CliOptions {
    pub incantation: Incantation,
    pub cli_config: UserConfig,
    pub config_file: Option<PathBuf>,
    pub credentials_file: Option<PathBuf>,
    pub merge_options: MergeOptions,
}

pub fn parse_arguments<I, S>(args: I) -> Result<CliOptions, clap::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let matches = clap::App::new(DID_IT_RUN_NAME)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name(CONFIG_FILE)
                .long("config")
                .value_name("FILE")
                .help("Path to config file"),
        )
        .arg(
            Arg::with_name(CREDENTIALS_FILE)
                .long("credentials")
                .value_name("FILE")
                .help("Path to credentials file"),
        )
        .arg(
            Arg::with_name(NO_EMAIL)
                .long("no-email")
                .help("Do not send email notifications")
                .conflicts_with(EMAIL),
        )
        .arg(
            Arg::with_name(NO_VALIDATE)
                .long("no-validate")
                .help("Do not validate credentials and inputs"),
        )
        .arg(
            Arg::with_name(TIMEOUT)
                .long("timeout")
                .help("Timeout in seconds")
                .number_of_values(1)
                .validator(validate_timeout),
        )
        .arg(
            Arg::with_name(EMAIL)
                .short("e")
                .long("email")
                .help("Email address(es) to receive notifications")
                .number_of_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name(COMMAND)
                .help("Command to run")
                .required(true),
        )
        .arg(
            Arg::with_name(ARGUMENTS)
                .help(&format!("{} arguments", COMMAND))
                .min_values(0),
        )
        .get_matches_from_safe(args)?;
    // Command is a required argument. We return an Err before reaching this
    // point if it is not provided.
    let command = matches.value_of_os(COMMAND).unwrap();
    let args = matches.values_of_os(ARGUMENTS).unwrap_or_default();
    let incantation = Incantation::new(command, args);
    let mut cli_config: UserConfig = Default::default();
    if let Some(recipients) = matches.values_of_lossy(EMAIL) {
        cli_config.email = Some(EmailConfig { recipients });
    }
    cli_config.validate = Some(!matches.is_present(NO_VALIDATE));
    if let Some(timeout) = matches.value_of(TIMEOUT) {
        // Clap already validates this value using `validate_timeout`.
        cli_config.timeout = Some(timeout.parse().unwrap());
    }
    let config_file = matches.value_of_os(CONFIG_FILE).map(PathBuf::from);
    let credentials_file =
        matches.value_of_os(CREDENTIALS_FILE).map(PathBuf::from);
    let merge_options = MergeOptions {
        no_email: matches.is_present(NO_EMAIL),
    };
    Ok(CliOptions {
        incantation,
        cli_config,
        config_file,
        credentials_file,
        merge_options,
    })
}

fn validate_timeout(timeout: String) -> Result<(), String> {
    match timeout.parse::<TimeoutInput>() {
        Ok(_) => Ok(()),
        Err(err) => Err(format!(
            "Cannot parse integer timeout value \"{}\": {}",
            timeout, err
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::ErrorKind;

    const BINARY_NAME: &str = "diditrun";

    #[test]
    fn returns_error_with_missing_command() {
        let args = [BINARY_NAME];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn returns_error_with_bad_flag() {
        let args = [BINARY_NAME, "--some-flag"];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::UnknownArgument);
    }

    #[test]
    fn returns_help_with_help_flag() {
        for help_flag in &["-h", "--help"] {
            let args = [BINARY_NAME, help_flag];
            let result = parse_arguments(&args);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind, ErrorKind::HelpDisplayed);
        }
    }

    #[test]
    fn returns_version_with_version_flag() {
        for version_flag in &["-V", "--version"] {
            let args = [BINARY_NAME, version_flag];
            let result = parse_arguments(&args);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind, ErrorKind::VersionDisplayed);
        }
    }

    #[test]
    fn parses_config_file_argument() {
        let file = "path/to/file";
        let args = [BINARY_NAME, "--config", file, "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().config_file, Some(PathBuf::from(file)));
    }

    #[test]
    fn returns_error_with_no_config_file_argument() {
        let args = [BINARY_NAME, "--config", "command"];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
        let result = parse_arguments(&[BINARY_NAME, "command", "--config"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::EmptyValue);
    }

    #[test]
    fn parses_credentials_file_argument() {
        let file = "path/to/file";
        let args = [BINARY_NAME, "--credentials", file, "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().credentials_file, Some(PathBuf::from(file)));
    }

    #[test]
    fn returns_error_with_no_credentials_file_argument() {
        let args = [BINARY_NAME, "--credentials", "command"];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
        let result = parse_arguments(&[BINARY_NAME, "command", "--config"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::EmptyValue);
    }

    #[test]
    fn removes_validation_with_no_validate_flag() {
        let args = [BINARY_NAME, "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert!(result.unwrap().cli_config.validate.unwrap());
        let args = [BINARY_NAME, "--no-validate", "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert!(!result.unwrap().cli_config.validate.unwrap());
    }

    #[test]
    fn configures_timeout_with_timeout_option() {
        let timeout = 60;
        let args = [BINARY_NAME, "--timeout", &timeout.to_string(), "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cli_config.timeout, Some(timeout));
    }

    #[test]
    fn omits_timeout_withut_timeout_option() {
        let args = [BINARY_NAME, "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().cli_config.timeout, None);
    }

    #[test]
    fn returns_error_with_bad_timeout_argument() {
        let invalid_timeout_values = ["ten", "3.14"];
        for timeout in &invalid_timeout_values {
            let args = [BINARY_NAME, "--timeout", timeout, "command"];
            let result = parse_arguments(&args);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind, ErrorKind::ValueValidation);
        }
    }

    #[test]
    fn returns_error_with_no_timeout_argument() {
        let args = [BINARY_NAME, "--timeout", "command"];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
        let result = parse_arguments(&[BINARY_NAME, "command", "--timeout"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::EmptyValue);
    }

    #[test]
    fn returns_error_with_unspecified_email() {
        let args = [BINARY_NAME, "--email", "command"];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
        let result = parse_arguments(&[BINARY_NAME, "command", "--email"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::EmptyValue);
    }

    #[test]
    fn specifying_no_email_flag_adds_config() {
        let args = [BINARY_NAME, "--no-email", "command"];
        let result = parse_arguments(&args);
        assert!(result.unwrap().merge_options.no_email);
    }

    #[test]
    fn not_no_email_flag_does_not_add_config() {
        let args = [BINARY_NAME, "command"];
        let result = parse_arguments(&args);
        assert!(!result.unwrap().merge_options.no_email);
    }

    #[test]
    fn specifying_email_conflicts_with_no_email_flag() {
        let args = [
            BINARY_NAME,
            "--email",
            "someone@something.com",
            "--no-email",
            "command",
        ];
        let result = parse_arguments(&args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::ArgumentConflict);
    }

    #[test]
    fn parses_command_with_no_arguments() {
        let args = [BINARY_NAME, "command"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        let incantation = result.unwrap().incantation;
        assert_eq!(incantation.command, "command");
        assert!(incantation.args.is_empty());
    }

    #[test]
    fn parses_command_with_one_argument() {
        let args = [BINARY_NAME, "command", "foo"];
        let result = parse_arguments(&args);
        assert!(result.is_ok());
        let incantation = result.unwrap().incantation;
        assert_eq!(incantation.command, "command");
        assert_eq!(incantation.args, ["foo"]);
    }

    #[test]
    fn parses_command_with_many_arguments() {
        let args_variants = [
            vec![BINARY_NAME, "command", "foo", "bar", "baz"],
            vec![BINARY_NAME, "command", "--", "foo", "bar", "baz"],
            vec![BINARY_NAME, "command", "foo", "--", "bar", "baz"],
        ];
        for args in &args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let incantation = result.unwrap().incantation;
            assert_eq!(incantation.command, "command");
            assert_eq!(incantation.args, ["foo", "bar", "baz"]);
        }
    }

    #[test]
    fn parses_command_with_flag() {
        let args_variants = [
            [BINARY_NAME, "command", "arg", "--", "--flag"],
            [BINARY_NAME, "command", "--", "arg", "--flag"],
        ];
        for args in &args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let incantation = result.unwrap().incantation;
            assert_eq!(incantation.command, "command");
            assert_eq!(incantation.args, ["arg", "--flag"]);
        }
    }

    #[test]
    fn parses_command_with_email() {
        let args_variants = [
            [
                BINARY_NAME,
                "--email",
                "someone@example.com",
                "command",
                "foo",
                "bar",
            ],
            [
                BINARY_NAME,
                "-e",
                "someone@example.com",
                "command",
                "foo",
                "bar",
            ],
            [
                BINARY_NAME,
                "command",
                "--email",
                "someone@example.com",
                "foo",
                "bar",
            ],
            [
                BINARY_NAME,
                "command",
                "foo",
                "bar",
                "--email",
                "someone@example.com",
            ],
            [
                BINARY_NAME,
                "command",
                "foo",
                "--email",
                "someone@example.com",
                "bar",
            ],
        ];
        for args in &args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let options = result.unwrap();
            assert_eq!(options.incantation.command, "command");
            assert_eq!(options.incantation.args, ["foo", "bar"]);
            assert_eq!(
                options.cli_config.email.unwrap().recipients,
                ["someone@example.com"]
            );
        }
    }

    #[test]
    fn parses_command_with_multiple_emails() {
        let args_variants = [
            [
                BINARY_NAME,
                "--email",
                "someone@example.com",
                "--email",
                "someone_else@example.com",
                "command",
                "foo",
            ],
            [
                BINARY_NAME,
                "--email",
                "someone@example.com",
                "command",
                "--email",
                "someone_else@example.com",
                "foo",
            ],
        ];
        for args in &args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let options = result.unwrap();
            assert_eq!(options.incantation.command, "command");
            assert_eq!(options.incantation.args, ["foo"]);
            assert_eq!(
                options.cli_config.email.unwrap().recipients,
                ["someone@example.com", "someone_else@example.com"]
            );
        }
    }

    #[test]
    fn parses_command_with_email_and_argument_flags() {
        let args_variants = [
            [
                BINARY_NAME,
                "--email",
                "someone@example.com",
                "command",
                "arg",
                "--",
                "--flag",
            ],
            [
                BINARY_NAME,
                "command",
                "--email",
                "someone@example.com",
                "--",
                "arg",
                "--flag",
            ],
        ];
        for args in &args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let options = result.unwrap();
            assert_eq!(options.incantation.command, "command");
            assert_eq!(options.incantation.args, ["arg", "--flag"]);
            assert_eq!(
                options.cli_config.email.unwrap().recipients,
                ["someone@example.com"]
            );
        }
    }

    // Hacks to minimize coverage report errors
    #[test]
    fn maximize_coverage_report() {
        let options = CliOptions {
            incantation: Incantation::new("command", vec!["arg"]),
            cli_config: UserConfig::default(),
            config_file: None,
            credentials_file: None,
            merge_options: MergeOptions::default(),
        };
        let _ = format!("{:?}", options);
    }
}
