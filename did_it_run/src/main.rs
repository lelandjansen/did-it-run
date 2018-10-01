#[macro_use]
extern crate clap;

mod cli;
mod incantation;

use std::env;
use std::process;

fn main() {
    let result = cli::parse_arguments(env::args());
    let incantation = match result {
        Ok(incantation) => incantation,
        Err(err) => err.exit(),
    };
    let status = incantation::run(incantation);
    match status {
        Ok(status) => {
            process::exit(match status.code() {
                Some(code) => code,
                None => 0,
            });
        },
        Err(error) => {
            eprintln!("{}", error);
            process::exit(match error.raw_os_error() {
                Some(code) => code,
                None => 1,
            });
        },
    }
}
