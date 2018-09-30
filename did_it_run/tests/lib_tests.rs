extern crate did_it_run;

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
        let status = did_it_run::run_command("/usr/bin/env", args);
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
        let status = did_it_run::run_command(&"/usr/bin/env", &args);
        assert_eq!(argument_count, status.unwrap().code().unwrap());
        args.push("another_arg");
    }
}

#[test]
fn returns_error_with_bad_command() {
    let result = did_it_run::run_command("command-that-does-not-exit", vec![]);
    assert_eq!(std::io::ErrorKind::NotFound, result.unwrap_err().kind());
}
