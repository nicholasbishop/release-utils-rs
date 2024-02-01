// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use release_utils::cmd::run_cmd;
use release_utils::github::{self, Gh};
use release_utils::release::*;
use release_utils::{get_github_sha, Package, Repo};
use std::env;
use std::path::PathBuf;
use std::process::Command;

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
    let commit_message_subject =
        repo.get_commit_message_subject(&commit_sha)?;

    if !commit_message_subject.starts_with("release:") {
        println!("{commit_sha} does not contain the release trigger");
        return Ok(());
    }

    let lib_pkg = Package::new("release-utils");
    let bin_pkg = Package::new("auto-release");
    release_packages(&[lib_pkg, bin_pkg.clone()])?;

    create_github_release(&bin_pkg)
}

/// Create a new Github release for the package, if it does not already
/// exist. This release includes a prebuilt auto-release executable for
/// convenience.
fn create_github_release(pkg: &Package) -> Result<()> {
    let version = pkg.get_local_version()?;
    let tag = pkg.get_git_tag_name(&version);

    let gh = Gh::new();
    if gh.does_release_exist(&tag)? {
        println!("github release {tag} already exists");
        return Ok(());
    }

    // This executable is intended to run in the default Github Actions
    // Ubuntu runner, i.e. the same environment we're building in, so
    // don't bother with anything clever like musl.
    let mut cmd = Command::new("cargo");
    cmd.args(["build", "--package", "auto-release", "--release"]);
    run_cmd(cmd)?;
    let exe_path = PathBuf::from("target/release/auto-release");

    // Strip the executable to reduce size. In future Rust releases this
    // won't be needed, see
    // https://kobzol.github.io/rust/cargo/2024/01/23/making-rust-binaries-smaller-by-default.html
    let mut cmd = Command::new("strip");
    cmd.arg(&exe_path);
    run_cmd(cmd)?;

    gh.create_release(github::CreateRelease {
        tag: tag.clone(),
        title: Some(tag),
        notes: None,
        files: vec![exe_path],
    })?;

    Ok(())
}
