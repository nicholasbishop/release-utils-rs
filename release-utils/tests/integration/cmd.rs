// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::cmd::*;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
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

    if let RunCommandError::NonZeroExit { cmd, status } =
        run_cmd(Command::new("false")).unwrap_err()
    {
        assert_eq!(cmd, "false");
        assert!(!status.success());
    } else {
        panic!("incorrect error type");
    }

    if let RunCommandError::Launch { cmd, .. } =
        run_cmd(Command::new("does-not-exist")).unwrap_err()
    {
        assert_eq!(cmd, "does-not-exist");
    } else {
        panic!("incorrect error type");
    }
}

#[test]
fn test_get_cmd_stdout() {
    let mut cmd = Command::new("echo");
    cmd.arg("hello world");
    assert_eq!(get_cmd_stdout(cmd).unwrap(), b"hello world\n");

    if let RunCommandError::NonZeroExit { cmd, status } =
        get_cmd_stdout(Command::new("false")).unwrap_err()
    {
        assert_eq!(cmd, "false");
        assert!(!status.success());
    } else {
        panic!("incorrect error type");
    }

    if let RunCommandError::Launch { cmd, .. } =
        get_cmd_stdout(Command::new("does-not-exist")).unwrap_err()
    {
        assert_eq!(cmd, "does-not-exist");
    } else {
        panic!("incorrect error type");
    }
}

#[test]
fn test_get_cmd_stdout_utf8() {
    let mut cmd = Command::new("echo");
    cmd.arg("hello world");
    assert_eq!(get_cmd_stdout_utf8(cmd).unwrap(), "hello world\n");

    let mut cmd = Command::new("echo");
    cmd.arg(OsStr::from_bytes(b"\xff"));
    assert!(get_cmd_stdout_utf8(cmd).is_err());
}

#[test]
fn test_cmd_error_display() {
    assert_eq!(
        run_cmd(Command::new("false")).unwrap_err().to_string(),
        r#"command "false" failed with exit status: 1"#
    );

    let msg = run_cmd(Command::new("does-not-exist"))
        .unwrap_err()
        .to_string();
    assert_eq!(msg, r#"failed to launch command "does-not-exist""#);

    let mut cmd = Command::new("echo");
    cmd.arg(OsStr::from_bytes(b"\xff"));
    let msg = get_cmd_stdout_utf8(cmd).unwrap_err().to_string();
    // In older versions of Rust the "ff" is lowercase, in newer
    // versions it's uppercase. To make tests work either way, force it
    // to lowercase.
    let msg = msg.to_lowercase();
    assert_eq!(msg, r#"command "echo \xff" output is not utf-8"#);
}
