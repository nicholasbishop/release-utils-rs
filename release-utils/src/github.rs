// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tools for working with the Github API.

use crate::cmd::{run_cmd, RunCommandError};
use std::path::PathBuf;
use std::process::Command;

/// Wrapper for the [`gh`] tool.
///
/// This tool is already available and authenticated when running
/// running code in a Github Actions workflow.
///
/// [`gh`]: https://cli.github.com/
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Gh {
    exe: PathBuf,
}

impl Gh {
    /// Create a new `Gh`.
    pub fn new() -> Self {
        Self::with_exe(PathBuf::from("gh"))
    }

    /// Create a new `Gh` using `exe` as the path to the `gh` executable.
    pub fn with_exe(exe: PathBuf) -> Self {
        Self { exe }
    }

    /// Create a new release.
    pub fn create_release(&self, opt: CreateRelease) -> Result<(), RunCommandError> {
        let mut cmd = Command::new(&self.exe);
        cmd.args([
            "release",
            "create",
            // Abort if tag does not exist.
            "--verify-tag",
        ]);

        if let Some(title) = &opt.title {
            cmd.args(["--title", title]);
        }

        if let Some(notes) = &opt.notes {
            cmd.args(["--notes", notes]);
        }

        // Tag from which to create the release.
        cmd.arg(&opt.tag);

        // Add files to upload with the release.
        cmd.args(&opt.files);

        run_cmd(cmd)
    }

    /// Check if a release for the given `tag` exists.
    pub fn does_release_exist(&self, tag: &str) -> Result<bool, RunCommandError> {
        let mut cmd = Command::new(&self.exe);
        cmd.args(["release", "view", tag]);
        match run_cmd(cmd) {
            Ok(()) => Ok(true),
            Err(err @ RunCommandError::Launch { .. }) => Err(err),
            Err(err @ RunCommandError::NonUtf8 { .. }) => Err(err),
            Err(RunCommandError::NonZeroExit { cmd, status }) => {
                // There are probably other ways this could fail, but
                // checking for code 1 should be close enough.
                if status.code() == Some(1) {
                    Ok(false)
                } else {
                    Err(RunCommandError::NonZeroExit { cmd, status })
                }
            }
        }
    }
}

impl Default for Gh {
    fn default() -> Self {
        Self::new()
    }
}

/// Inputs for creating a Github release.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CreateRelease {
    /// Tag to create the release for. This tag must already exist
    /// before calling `execute`.
    pub tag: String,

    /// Release title.
    pub title: Option<String>,

    /// Release notes.
    pub notes: Option<String>,

    /// Files to upload and attach to the release.
    pub files: Vec<PathBuf>,
}
