#![deny(warnings)]

mod cli;
mod config;
mod duration_format;
mod exit_code;
mod incantation;
mod notifications;

use crate::config::{DEFAULT_CONFIG_FILES, DEFAULT_CREDENTIALS_FILES};
use crate::exit_code::ExitCode;
use crate::notifications::Notifier;
use std::env;
use std::fmt::Display;
use std::process;

const DID_IT_RUN_NAME: &str = "Did it Run?";
const DID_IT_RUN_EMAIL: &str = "notifications@didit.run";

fn exit<E: Display>(err: E, exit_code: ExitCode) -> ! {
    eprintln!("{}", err);
    process::exit(exit_code);
}

fn main() {
    let options =
        cli::parse_arguments(env::args_os()).unwrap_or_else(|err| err.exit());
    let file_config: config::UserConfig =
        config::load_file(options.config_file, DEFAULT_CONFIG_FILES.to_vec())
            .unwrap_or_else(|err| exit(err, exit_code::CONFIG));
    let user_config =
        config::merge(options.cli_config, file_config, options.merge_options);
    let config = config::Config::from_user_config(user_config)
        .unwrap_or_else(|err| exit(err, exit_code::CONFIG));
    let user_credentials: config::UserCredentials = config::load_file(
        options.credentials_file,
        DEFAULT_CREDENTIALS_FILES.to_vec(),
    )
    .unwrap_or_else(|err| exit(err, exit_code::CONFIG));
    let credentials =
        config::Credentials::from_user_credentials(user_credentials)
            .unwrap_or_else(|err| exit(err, exit_code::CONFIG));
    let mut notifier = Notifier::new(config, credentials)
        .unwrap_or_else(|err| exit(err, exit_code::FAILURE));

    let outcome = incantation::run(&options.incantation);
    let incantation_exit_code = match outcome.result {
        Ok(status) => status.code().unwrap_or(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("{}", err);
            err.raw_os_error().unwrap_or(exit_code::FAILURE)
        },
    };
    let event = notifications::Event::Finished {
        incantation: options.incantation,
        exit_code: incantation_exit_code,
        elapsed_time: outcome.elapsed_time,
    };

    notifier
        .notify(event)
        .unwrap_or_else(|err| exit(err, exit_code::FAILURE));

    process::exit(incantation_exit_code);
}

#[cfg(test)]
mod test {
    use super::*;
    use lazy_static::lazy_static;
    use std::collections::HashSet;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    lazy_static! {
        static ref PROJECT_ROOT_NAME: OsString = OsString::from("did-it-run");
        pub static ref PROJECT_ROOT_PATH: PathBuf = {
            let pwd =
                env::current_dir().expect("Current directory not available.");
            let mut path = PathBuf::from(pwd);
            // Heuristic: Use a unique set of file names contained in the
            // project's root folder to identify it.
            let root_folder_files_sample: HashSet<_> =
                ["did_it_run", "Cargo.lock", "README.md"]
                    .iter()
                    .map(OsString::from)
                    .collect();
            loop {
                let files: HashSet<_> = fs::read_dir(&path)
                    .unwrap()
                    .map(|file| file.unwrap().file_name())
                    .collect();
                if files.is_superset(&root_folder_files_sample) {
                    break;
                }
                if path == PathBuf::from("/") {
                    panic!("Test not run inside project directory.");
                }
                path.pop();
            }
            path
        };
    }
}
