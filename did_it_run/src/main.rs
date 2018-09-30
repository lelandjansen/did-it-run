extern crate did_it_run;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }
    let status = did_it_run::run_command(&args[1], &args[2..]);
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
