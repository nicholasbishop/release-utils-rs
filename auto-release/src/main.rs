// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(unsafe_code)]

mod args;

use anyhow::Result;
use args::{parse_args, Cli, Condition};
use release_utils::release::release_packages;
use release_utils::{get_github_sha, Package, Repo};

fn check_condition(condition: Condition) -> Result<bool> {
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
        println!("commit message {msg_kind} of {commit_sha} does not start with \"{prefix}\"");
        Ok(false)
    }
}

fn execute(cli: Cli) -> Result<()> {
    if let Some(condition) = cli.condition {
        if !check_condition(condition)? {
            return Ok(());
        }
    }

    let packages: Vec<_> = cli.package.iter().map(Package::new).collect();

    Ok(release_packages(&packages)?)
}

fn main() -> Result<()> {
    let cli = parse_args();

    execute(cli)
}
