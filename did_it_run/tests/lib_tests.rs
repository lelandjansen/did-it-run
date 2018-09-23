extern crate did_it_run;

mod common;

use common::fixtures_command;
use std::io;

#[test]
fn exits_with_status() {
    let command = fixtures_command("exit-with-status.sh");
    for status in vec![-7, -1, 0, 1, 2, 254, 255, 256, 2000] {
        // When a parent uses retrieves the exit status of its child, only the
        // least-significant eight bits are available.
        let status = status & 0xFF;
        let result = did_it_run::run_command(&command, &[status.to_string()]);
        assert_eq!(status, result.unwrap().code().unwrap());
    }
}

#[test]
fn handles_arguments() {
    let command = fixtures_command("exit-with-argument-count.sh");
    for argument_count in 0..5 {
        let args: Vec<String> = (0..argument_count)
            .into_iter()
            .map(|item| item.to_string())
            .collect();
        let result = did_it_run::run_command(&command, &args);
        assert_eq!(argument_count, result.unwrap().code().unwrap());
    }
}

#[test]
fn returns_error_with_bad_command() {
    let command = fixtures_command("this-command-does-not-exist");
    let result = did_it_run::run_command(&command, &[]);
    assert_eq!(io::ErrorKind::NotFound, result.unwrap_err().kind());
}
