// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Parse command-line arguments.
//!
//! This executable has a very simple interface, so we can implement it
//! manually rather than using a full-featured crate like `clap`. This
//! improves from-scratch compilation time, which matters for `cargo
//! install`.

use std::{env, process};

#[derive(Debug, Eq, PartialEq)]
pub enum Condition {
    Body,
    Subject,
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct Cli {
    pub package: Vec<String>,
    pub condition: Option<Condition>,
}

const USAGE: &str = r#"Usage:
auto-release -p <PKG> [-p <PKG>...] [--condition body|subject]

Options:
  -p, --package <PACKAGE>
      --condition <CONDITION>  [possible values: body, subject]
  -h, --help                   Print help
"#;

enum ArgState {
    Any,
    Package,
    Condition,
}

#[derive(Debug, Eq, PartialEq)]
enum ArgParseResult {
    Success(Cli),
    ShowUsage,
    InvalidArg,
    InvalidCondition,
    MissingValue,
    MissingPackage,
}

/// Parse arguments from a `String` iterator.
fn parse_args_from_iter(
    mut args: impl Iterator<Item = String>,
) -> ArgParseResult {
    let mut cli = Cli::default();

    let mut arg_state = ArgState::Any;
    // Skip the first arg, name of program.
    args.next();

    for arg in args {
        match arg_state {
            ArgState::Any => {
                if arg == "-p" || arg == "--package" {
                    arg_state = ArgState::Package;
                } else if arg == "--condition" {
                    arg_state = ArgState::Condition;
                } else if arg == "-h" || arg == "--help" {
                    return ArgParseResult::ShowUsage;
                } else {
                    return ArgParseResult::InvalidArg;
                }
            }
            ArgState::Package => {
                cli.package.push(arg);
                arg_state = ArgState::Any;
            }
            ArgState::Condition => {
                if arg == "body" {
                    cli.condition = Some(Condition::Body);
                } else if arg == "subject" {
                    cli.condition = Some(Condition::Subject);
                } else {
                    return ArgParseResult::InvalidCondition;
                }
                arg_state = ArgState::Any;
            }
        }
    }

    if !matches!(arg_state, ArgState::Any) {
        return ArgParseResult::MissingValue;
    }

    if cli.package.is_empty() {
        return ArgParseResult::MissingPackage;
    }

    ArgParseResult::Success(cli)
}

/// Parse command-line arguments.
pub fn parse_args() -> Cli {
    let err = match parse_args_from_iter(env::args()) {
        ArgParseResult::Success(cli) => {
            return cli;
        }
        ArgParseResult::ShowUsage => {
            print!("{USAGE}");
            process::exit(0);
        }
        ArgParseResult::InvalidArg => "invalid arg",
        ArgParseResult::InvalidCondition => "invalid condition",
        ArgParseResult::MissingValue => "missing arg value",
        ArgParseResult::MissingPackage => {
            "at least one package must be specified"
        }
    };

    println!("error: {err}");
    print!("{USAGE}");
    process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &[&str]) -> impl Iterator<Item = String> {
        s.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn test_arg_parse() {
        assert_eq!(
            parse_args_from_iter(args(&[])),
            ArgParseResult::MissingPackage
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release"])),
            ArgParseResult::MissingPackage
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "-a"])),
            ArgParseResult::InvalidArg
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "-p"])),
            ArgParseResult::MissingValue
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "--condition"])),
            ArgParseResult::MissingValue
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "--condition", "foo"])),
            ArgParseResult::InvalidCondition
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "-p", "foo"])),
            ArgParseResult::Success(Cli {
                package: vec!["foo".to_string()],
                condition: None,
            })
        );

        assert_eq!(
            parse_args_from_iter(args(&[
                "auto-release",
                "-p",
                "foo",
                "--package",
                "bar"
            ])),
            ArgParseResult::Success(Cli {
                package: vec!["foo".to_string(), "bar".to_string()],
                condition: None,
            })
        );

        assert_eq!(
            parse_args_from_iter(args(&[
                "auto-release",
                "-p",
                "foo",
                "--condition",
                "body"
            ])),
            ArgParseResult::Success(Cli {
                package: vec!["foo".to_string()],
                condition: Some(Condition::Body),
            })
        );

        assert_eq!(
            parse_args_from_iter(args(&[
                "auto-release",
                "-p",
                "foo",
                "--condition",
                "subject"
            ])),
            ArgParseResult::Success(Cli {
                package: vec!["foo".to_string()],
                condition: Some(Condition::Subject),
            })
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "-h"])),
            ArgParseResult::ShowUsage
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "--help"])),
            ArgParseResult::ShowUsage
        );

        assert_eq!(
            parse_args_from_iter(args(&["auto-release", "-h", "-p"])),
            ArgParseResult::ShowUsage
        );
    }
}
