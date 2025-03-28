// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

mod args;

use args::{Cli, Condition, parse_args};
use release_utils::release::release_packages;
use release_utils::{Package, Repo, get_github_sha};
use std::process;

type Error = Box<dyn std::error::Error>;

fn check_condition(condition: Condition) -> Result<bool, Error> {
    let commit_sha = get_github_sha()?;
    let repo = Repo::open()?;

    let prefix = "release:";

    let msg_text;
    let msg_kind;

    match condition {
        Condition::Body => {
            msg_text = repo.get_commit_message_body(&commit_sha)?;
            msg_kind = "body";
        }
        Condition::Subject => {
            msg_text = repo.get_commit_message_subject(&commit_sha)?;
            msg_kind = "subject";
        }
    }

    if msg_text.starts_with(prefix) {
        Ok(true)
    } else {
        println!(
            "commit message {msg_kind} of {commit_sha} does not start with \"{prefix}\""
        );
        Ok(false)
    }
}

fn execute(cli: Cli) -> Result<(), Error> {
    if let Some(condition) = cli.condition {
        if !check_condition(condition)? {
            return Ok(());
        }
    }

    let packages: Vec<_> = cli.package.iter().map(Package::new).collect();

    Ok(release_packages(&packages)?)
}

fn main() {
    let cli = parse_args();

    if let Err(err) = execute(cli) {
        println!("{err}");
        println!("Caused by:");
        let mut err = &*err;
        while let Some(cause) = err.source() {
            println!("    {cause}");
            err = cause;
        }

        process::exit(1);
    }
}
