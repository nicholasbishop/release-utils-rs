// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::cmd::*;
use std::process::Command;

#[test]
fn test_format_cmd() {
    assert_eq!(format_cmd(&Command::new("echo")), "echo");
    assert_eq!(format_cmd(&Command::new("echo").arg("hello")), "echo hello");
    assert_eq!(
        format_cmd(&Command::new("echo").arg("hello world")),
        "echo hello world"
    );
}

#[test]
fn test_run_cmd() {
    assert!(run_cmd(Command::new("true")).is_ok());
    assert!(!run_cmd(Command::new("false")).is_ok());
    assert!(!run_cmd(Command::new("does-not-exist")).is_ok());
}

#[test]
fn test_get_cmd_stdout() {
    let mut cmd = Command::new("echo");
    cmd.arg("hello world");
    assert_eq!(get_cmd_stdout(cmd).unwrap(), b"hello world\n");

    assert!(!get_cmd_stdout(Command::new("false")).is_ok());
    assert!(!get_cmd_stdout(Command::new("does-not-exist")).is_ok());
}
