// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cmd::{get_cmd_stdout, RunCommandError};
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Temporary directory. The directory and its contents are
/// automatically deleted when dropped.
pub struct TempDir(PathBuf);

impl TempDir {
    /// Create a new temporary directory.
    pub fn new() -> Result<Self, RunCommandError> {
        let mut cmd = Command::new("mktemp");
        cmd.arg("--directory");
        let mut output = get_cmd_stdout(cmd)?;
        // Drop the trailing newline.
        output.pop();
        let path = PathBuf::from(OsString::from_vec(output));
        Ok(Self(path))
    }

    /// Get the temporary path.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
