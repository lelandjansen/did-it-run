use std::borrow::Cow;
use std::convert::Into;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Incantation {
    pub command: OsString,
    pub args: Vec<OsString>,
}

impl Incantation {
    pub fn new<S, I>(command: S, args: I) -> Self
    where
        S: Into<OsString>,
        I: IntoIterator<Item = S>,
    {
        Incantation {
            command: command.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

impl fmt::Display for Incantation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args = self
            .args
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<Cow<str>>>();
        write!(f, "{} {}", self.command.to_string_lossy(), args.join(" "))
    }
}

pub struct IncantationOutcome {
    pub result: io::Result<ExitStatus>,
    pub elapsed_time: Duration,
}

pub fn run(incantation: &Incantation) -> IncantationOutcome {
    let now = Instant::now();
    let result = Command::new(incantation.command.clone())
        .args(incantation.args.clone())
        .spawn()
        .and_then(|mut child| child.wait());
    let elapsed_time = now.elapsed();
    IncantationOutcome {
        result,
        elapsed_time,
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
        let status_codes = [-7, -1, 0, 1, 2, 254, 255, 256, 2000];
        for status_code in &status_codes {
            let status_arg = status_code.to_string();
            let mut args = base_args.clone();
            args.push(&status_arg);
            let incantation = Incantation::new("/usr/bin/env", args.clone());
            let outcome = run(&incantation);
            // When a parent retrieves the exit status of its child, only the
            // least-significant eight bits are available.
            let status_code = status_code & 0xFF;
            assert_eq!(status_code, outcome.result.unwrap().code().unwrap());
        }
    }

    #[test]
    fn handles_arguments() {
        let mut args = vec!["bash", "-c", EXIT_WITH_ARGUMENT_COUNT, "bash"];
        for argument_count in 0..5 {
            let incantation = Incantation::new("/usr/bin/env", args.clone());
            let outcome = run(&incantation);
            assert_eq!(argument_count, outcome.result.unwrap().code().unwrap());
            args.push("another_arg");
        }
    }

    #[test]
    fn returns_error_with_bad_command() {
        let incantation = Incantation::new("wingardium-leviosa", vec![]);
        let outcome = run(&incantation);
        // It's leviOsa, not leviosA
        assert_eq!(io::ErrorKind::NotFound, outcome.result.unwrap_err().kind());
    }

    // Hacks to minimize coverage report errors
    #[test]
    fn maximize_coverage() {
        let incantation = Incantation {
            command: OsString::from("command"),
            args: vec![OsString::from("foo"), OsString::from("bar")],
        };
        let _ = format!("{:?}", incantation);
    }
}
