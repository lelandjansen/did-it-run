mod common;

use common::fixtures_command;
use std::process::{Command, Stdio};
use std::sync::Once;

const DOCKER_CONTAINER_TAG: &str = "did-it-run";
const DOCKER_FILE_DIRECTORY: &str = ".";
static SETUP: Once = Once::new();

fn build_container() {
    SETUP.call_once(|| {
        let status = Command::new("docker")
            .arg("build")
            .args(&["--tag", DOCKER_CONTAINER_TAG])
            .arg(DOCKER_FILE_DIRECTORY)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to build Docker container.");
        assert!(status.success());
    });
}

fn docker_run() -> Command {
    let mut command = Command::new("docker");
    command.arg("run");
    command.arg("--tty");
    command.args(&["--rm", DOCKER_CONTAINER_TAG]);
    command
}

#[test]
fn runs_command_with_several_arguments() {
    build_container();
    let output = docker_run()
        .args(&["echo", "Hello,", "world!"])
        .output()
        .expect("Failed to run command in Docker container.");
    assert_eq!(
        "Hello, world!",
        String::from_utf8_lossy(&output.stdout).trim()
    );
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn runs_command_with_one_argument() {
    build_container();
    let output = docker_run()
        .arg("echo")
        .output()
        .expect("Failed to run command in Docker container.");
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn exits_with_no_arguments() {
    build_container();
    let output = docker_run()
        .output()
        .expect("Failed to run command in Docker container.");
    assert!(String::from_utf8_lossy(&output.stdout).is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).is_empty());
}

#[test]
fn exits_with_child_status() {
    build_container();
    for status_code in 0..3 {
        let status = docker_run()
            .arg(fixtures_command("exit-with-status.sh"))
            .arg(status_code.to_string())
            .status()
            .expect("Failed to run command in Docker container.");
        assert_eq!(status_code, status.code().unwrap());
    }
}
