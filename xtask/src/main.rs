// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use release_utils::release::*;
use release_utils::{Package, Repo};
use std::env;

fn main() -> Result<()> {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("expected one argument");
    }

    let action = &args[1];
    if action == "release" {
        auto_release()
    } else {
        panic!("invalid action: {action}");
    }
}

/// Entry point for the auto-release process. This is intended to be run
/// from a Github Actions workflow.
fn auto_release() -> Result<()> {
    let commit_sha = get_github_sha()?;
    let repo = Repo::open()?;
    let commit_message_subject = repo.get_commit_message_subject(&commit_sha)?;

    if !commit_message_subject.starts_with("release:") {
        println!("{commit_sha} does not contain the release trigger");
        return Ok(());
    }

    release_packages(&[Package::new("release-utils")])
}
