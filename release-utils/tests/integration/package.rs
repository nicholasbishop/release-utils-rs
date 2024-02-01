// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::cmd::*;
use release_utils::{GetLocalVersionError, Package};
use std::env;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_package() {
    let pkg = Package::new("foo");
    assert_eq!(pkg.name(), "foo");
    assert_eq!(pkg.workspace(), env::current_dir().unwrap());
    assert_eq!(pkg.get_git_tag_name("1.2.3"), "foo-v1.2.3");
}

#[test]
fn test_package_local_version() {
    let tmp_dir = TempDir::new().unwrap();
    let mut cmd = Command::new("cargo");
    cmd.args(["init", "--name", "foo"]);
    cmd.arg(tmp_dir.path());
    run_cmd(cmd).unwrap();

    let pkg = Package::with_workspace("foo", tmp_dir.path());
    assert_eq!(pkg.get_local_version().unwrap(), "0.1.0");

    let pkg = Package::with_workspace("invalid", tmp_dir.path());
    if let GetLocalVersionError::PackageNotFound(name) =
        pkg.get_local_version().unwrap_err()
    {
        assert_eq!(name, "invalid");
    } else {
        panic!("unexpected error");
    }
}
