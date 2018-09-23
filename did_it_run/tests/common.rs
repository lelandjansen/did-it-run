use std::path::Path;

const FIXTURES_PATH: &str = "tests/fixtures";

pub fn fixtures_command(command: &str) -> String {
    Path::new(FIXTURES_PATH)
        .join(command)
        .to_str()
        .unwrap()
        .to_string()
}
