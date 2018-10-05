#[macro_use]
extern crate clap;
extern crate dirs;
#[macro_use]
extern crate lazy_static;
extern crate lettre;
extern crate lettre_email;
#[cfg(test)]
extern crate mailin_embedded;
#[cfg(test)]
extern crate mailparse;
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate native_tls;
extern crate semver;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod cli;
mod config;
mod duration_format;
mod email;
mod exit_code;
mod incantation;

use config::{DEFAULT_CONFIG_FILES, DEFAULT_CREDENTIALS_FILES};
use email::{Mailer, NotificationInfo};
use exit_code::ExitCode;
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
    let mailer = match (&config.email, credentials.smtp) {
        (Some(_), Some(credentials)) => {
            match Mailer::new(config, credentials) {
                Ok(mailer) => Some(mailer),
                Err(err) => exit(err, exit_code::FAILURE),
            }
        },
        (Some(_), None) => {
            exit("Cannot find SMTP credentials", exit_code::CONFIG);
        },
        (..) => None,
    };

    let outcome = incantation::run(&options.incantation);
    let incantation_exit_code = match outcome.result {
        Ok(status) => status.code().unwrap_or(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("{}", err);
            err.raw_os_error().unwrap_or(exit_code::FAILURE)
        },
    };
    let info = NotificationInfo {
        incantation: options.incantation,
        exit_code: incantation_exit_code,
        elapsed_time: outcome.elapsed_time,
    };

    if let Some(mut mailer) = mailer {
        if let Err(err) = mailer.notify(&info) {
            let message = format!("Failed to send email notification: {}", err);
            exit(message, exit_code::FAILURE);
        }
    }
    process::exit(incantation_exit_code);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ffi::OsString;
    use std::path::PathBuf;

    lazy_static! {
        static ref PROJECT_ROOT_NAME: OsString = OsString::from("did-it-run");
        pub static ref PROJECT_ROOT_PATH: PathBuf = {
            let mut path = PathBuf::from(env::var_os("PWD").unwrap());
            path.iter()
                .find(|component| component == &PROJECT_ROOT_NAME.as_os_str())
                .expect("Test not run in project directory");
            while path.file_name().unwrap() != PROJECT_ROOT_NAME.as_os_str() {
                path.pop();
            }
            path
        };
    }
}
