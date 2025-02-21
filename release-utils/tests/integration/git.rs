// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::Repo;
use release_utils::cmd::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_repo_open() {
    let tmp_dir = TempDir::new().unwrap();

    assert!(Repo::open_path(tmp_dir.path()).is_err());

    // Create a temporary repo.
    let mut cmd = Command::new("git");
    cmd.arg("init");
    cmd.arg(tmp_dir.path());
    run_cmd(cmd).unwrap();

    assert!(Repo::open_path(tmp_dir.path()).is_ok());
}

#[test]
fn test_get_commit_message() {
    let tmp_dir = TempDir::new().unwrap();

    // Create a temporary repo.
    let mut cmd = Command::new("git");
    cmd.arg("init");
    cmd.arg(tmp_dir.path());
    run_cmd(cmd).unwrap();

    // Configure identity.
    let mut cmd = Command::new("git");
    cmd.arg("-C");
    cmd.arg(tmp_dir.path());
    cmd.args(["config", "user.email", "release-utils-test@example.com"]);
    run_cmd(cmd).unwrap();
    let mut cmd = Command::new("git");
    cmd.arg("-C");
    cmd.arg(tmp_dir.path());
    cmd.args(["config", "user.name", "Release Utils Test"]);
    run_cmd(cmd).unwrap();

    // Create an empty commit with a known commit message.
    let mut cmd = Command::new("git");
    cmd.arg("-C");
    cmd.arg(tmp_dir.path());
    cmd.args([
        "commit",
        "--allow-empty",
        "-m",
        "hello world!\n\nHere's the body.",
    ]);
    run_cmd(cmd).unwrap();

    let repo = Repo::open_path(tmp_dir.path()).unwrap();

    assert_eq!(
        repo.get_commit_message_subject("HEAD").unwrap(),
        "hello world!"
    );
    assert_eq!(
        repo.get_commit_message_body("HEAD").unwrap(),
        "Here's the body.\n"
    );
}
