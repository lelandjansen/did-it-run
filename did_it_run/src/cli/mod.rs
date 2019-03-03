extern crate clap;

use clap::Arg;
use incantation::Incantation;
use std::ffi::{OsStr, OsString};

const COMMAND: &str = "COMMAND";
const ARGUMENTS: &str = "ARGUMENTS";

/// Parses the provided command line arguments. The first argument is the binary
/// name.
pub fn parse_arguments<I, S>(
    args: I,
) -> Result<Incantation<Vec<OsString>, OsString>, clap::Error>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString> + Clone,
{
    let matches = clap::App::new("Did it Run?")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name(COMMAND)
                .help("Command to run")
                .index(1)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGUMENTS)
                .help(&format!("{} arguments", COMMAND))
                .index(2)
                .min_values(0),
        )
        .get_matches_from_safe(args)?;
    let command = matches.value_of_os(COMMAND).unwrap().to_os_string();
    let args = match matches.values_of_os(ARGUMENTS) {
        Some(args) => args.map(OsStr::to_os_string).collect(),
        None => vec![],
    };
    Ok(Incantation { command, args })
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::ErrorKind;

    const BINARY_NAME: &str = "diditrun";

    #[test]
    fn returns_error_with_missing_command() {
        let args = vec![BINARY_NAME];
        let result = parse_arguments(args);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().kind,
            ErrorKind::MissingRequiredArgument
        );
    }

    #[test]
    fn returns_error_with_bad_flag() {
        let args = vec![BINARY_NAME, "--some-flag"];
        let result = parse_arguments(args);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind, ErrorKind::UnknownArgument);
    }

    #[test]
    fn returns_help_with_help_flag() {
        for help_flag in vec!["-h", "--help"] {
            let args = vec![BINARY_NAME, help_flag];
            let result = parse_arguments(args);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind, ErrorKind::HelpDisplayed);
        }
    }

    #[test]
    fn returns_version_with_version_flag() {
        for version_flag in vec!["-V", "--version"] {
            let args = vec![BINARY_NAME, version_flag];
            let result = parse_arguments(args);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind, ErrorKind::VersionDisplayed);
        }
    }

    #[test]
    fn parses_command_with_no_arguments() {
        let args = vec![BINARY_NAME, "command"];
        let result = parse_arguments(args);
        assert!(result.is_ok());
        let incantation = result.unwrap();
        assert_eq!(incantation.command, "command");
        assert!(incantation.args.is_empty());
    }

    #[test]
    fn parses_command_with_one_argument() {
        let args = vec![BINARY_NAME, "command", "foo"];
        let result = parse_arguments(args);
        assert!(result.is_ok());
        let incantation = result.unwrap();
        assert_eq!(incantation.command, "command");
        assert_eq!(incantation.args, vec!["foo"]);
    }

    #[test]
    fn parses_command_with_many_arguments() {
        let args_variants = vec![
            vec![BINARY_NAME, "command", "foo", "bar", "baz"],
            vec![BINARY_NAME, "command", "--", "foo", "bar", "baz"],
            vec![BINARY_NAME, "command", "foo", "--", "bar", "baz"],
        ];
        for args in args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let incantation = result.unwrap();
            assert_eq!(incantation.command, "command");
            assert_eq!(incantation.args, vec!["foo", "bar", "baz"]);
        }
    }

    #[test]
    fn parses_command_with_flag() {
        let args_variants = vec![
            vec![BINARY_NAME, "command", "arg", "--", "--flag"],
            vec![BINARY_NAME, "command", "--", "arg", "--flag"],
        ];
        for args in args_variants {
            let result = parse_arguments(args);
            assert!(result.is_ok());
            let incantation = result.unwrap();
            assert_eq!(incantation.command, "command");
            assert_eq!(incantation.args, vec!["arg", "--flag"]);
        }
    }
}
