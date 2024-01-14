// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::cmd::*;
use release_utils::Repo;
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
