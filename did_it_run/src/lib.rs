use std::ffi::OsStr;
use std::io;
use std::process::{Command, ExitStatus};

/// Executes `command` as a child process with the given `args`, waits for it to
/// finish, and returns the exit status.
///
/// Stdout, stderr, and stdin are inherited by the parent.
pub fn run_command<I, S>(command: S, args: I) -> io::Result<ExitStatus>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(command).args(args).spawn();
    match child {
        Ok(mut child) => child.wait(),
        Err(err) => Err(err),
    }
}
