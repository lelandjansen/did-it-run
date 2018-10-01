use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io;
use std::process::{Command, ExitStatus};

pub struct Incantation<I, S>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    pub command: S,
    pub args: I,
}

impl fmt::Debug for Incantation<Vec<OsString>, OsString> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?} {:?}", self.command, self.args)
    }
}

/// Executes the `incantation` as a child process, waits for it to finish, then
/// returns the exit status.
///
/// Stdout, stderr, and stdin are inherited by the parent.
pub fn run<I, S>(incantation: Incantation<I, S>) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    OsString: From<S>,
{
    let child = Command::new(incantation.command)
        .args(incantation.args)
        .spawn();
    match child {
        Ok(mut child) => child.wait(),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXIT_WITH_ARGUMENT_COUNT: &str = "exit $#";
    const EXIT_WITH_STATUS: &str = "exit $0";

    #[test]
    fn exits_with_status() {
        let base_args = vec!["bash", "-c", EXIT_WITH_STATUS];
        let status_codes = vec![-7, -1, 0, 1, 2, 254, 255, 256, 2000];
        for status_code in status_codes {
            let status_arg = status_code.to_string();
            let mut args = base_args.clone();
            args.push(&status_arg);
            let incantation = Incantation {
                command: "/usr/bin/env",
                args,
            };
            let status = run(incantation);
            // When a parent retrieves the exit status of its child, only the
            // least-significant eight bits are available.
            let status_code = status_code & 0xFF;
            assert_eq!(status_code, status.unwrap().code().unwrap());
        }
    }

    #[test]
    fn handles_arguments() {
        let mut args = vec!["bash", "-c", EXIT_WITH_ARGUMENT_COUNT, "bash"];
        for argument_count in 0..5 {
            let incantation = Incantation {
                command: "/usr/bin/env",
                args: args.clone(),
            };
            let status = run(incantation);
            assert_eq!(argument_count, status.unwrap().code().unwrap());
            args.push("another_arg");
        }
    }

    #[test]
    fn returns_error_with_bad_command() {
        let incantation = Incantation {
            command: "wingardium-leviosa", // It's leviOsa, not leviosA
            args: vec![],
        };
        let result = run(incantation);
        assert_eq!(io::ErrorKind::NotFound, result.unwrap_err().kind());
    }

    #[test]
    fn incantation_os_string_debug_format() {
        let incantation = Incantation {
            command: OsString::from("command"),
            args: vec![OsString::from("foo"), OsString::from("bar")],
        };
        assert_eq!(
            &format!("{:?}", incantation),
            "\"command\" [\"foo\", \"bar\"]"
        );
    }
}
