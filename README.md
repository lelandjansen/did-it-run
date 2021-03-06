# Did it Run?
[![Build Status][ci-badge]][ci]
[![Coverage][coverage-badge]][coverage]

Check-up on a command and get notified when it has finished running.

## tl;dr
```
$ diditrun --email someone@example.com some-long-running-command args

$ diditrun --help
Did it Run? 0.0.1
Leland Jansen <hello@lelandjansen.com>


USAGE:
    diditrun [FLAGS] [OPTIONS] <COMMAND> [--] [ARGUMENTS]...

FLAGS:
        --no-desktop     Do not show desktop notifications
        --no-email       Do not send email notifications
        --no-validate    Do not validate credentials and inputs
    -h, --help           Prints help information
    -V, --version        Prints version information

OPTIONS:
        --config <FILE>         Path to config file
        --credentials <FILE>    Path to credentials file
    -e, --email <EMAIL>...      Email address(es) to receive notifications
        --timeout <TIMEOUT>     Timeout in seconds

ARGS:
    <COMMAND>         Command to run
    <ARGUMENTS>...    COMMAND arguments
```

## Installation and setup

### Dependencies
* `libnotify` (Linux)

### Dev dependencies
* `libnotify-dev` (Linux)
* `libssl-dev` (Linux) 
* [OpenSSL][openssl-windows] (Windows)

Install the binary
```
$ cargo install --path did_it_run
```

Specify your SMTP server credentials in `~/.diditrun/credentials.toml` or
`~/diditrun/credentials.toml`. Example:
[diditrun/credentials.toml](tests/fixtures/diditrun/credentials.toml)

Optionally specify default configurations in `~/.diditrun/config.toml` or
`~/diditrun/config.toml`. Example:
[diditrun/config.toml](tests/fixtures/diditrun/config.toml)

## Testing
Create and self-sign a dummy TLS certificate (required by some email tests):
```
$ ./tests/fixtures/tls/make-cert.sh
$ sudo ./tests/fixtures/tls/install-cert.sh
```

Unit tests
```
$ cargo test --all
```

Test scripts (requires installation)
```
$ cargo install --path did_it_run
$ ./run-test-scripts.sh
```

Check code style (requires nightly)
```
$ ./check-style.sh
```

Apply suggested style changes (requires nightly)
```
$ ./beautify.sh
```

[ci]: https://dev.azure.com/lelandjansen/did-it-run/_build/latest?definitionId=1&branchName=master
[ci-badge]: https://dev.azure.com/lelandjansen/did-it-run/_apis/build/status/ci?branchName=master
[coverage]: https://codecov.io/gh/lelandjansen/did-it-run
[coverage-badge]: https://codecov.io/gh/lelandjansen/did-it-run/branch/master/graph/badge.svg
[openssl-windows]: https://slproweb.com/products/Win32OpenSSL.html
