// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for running `git` commands.

use crate::cmd::{get_cmd_stdout_utf8, run_cmd};
use anyhow::{anyhow, Result};
use std::env;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git repo.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Repo(PathBuf);

impl Repo {
    /// Get a `Repo` for the current directory.
    ///
    /// This will fail if the current directory does not contain a
    /// `.git` subdirectory.
    pub fn open() -> Result<Self> {
        let path = env::current_dir()?;
        Self::open_path(path)
    }

    /// Get a `Repo` for the given path.
    ///
    /// This will fail if the `path` does not contain a `.git`
    /// subdirectory.
    pub fn open_path<P>(path: P) -> Result<Self>
    where
        P: Into<PathBuf>,
    {
        let path = path.into();

        // Check that this is a git repo. This is just to help fail
        // quickly if the path is wrong; it isn't checking that the git
        // repo is valid or anything.
        //
        // Also, there are various special types of git checkouts so
        // it's quite possible this check is wrong for special
        // circumstances, but for a typical CI release process it should
        // be fine.
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Err(anyhow!("{} does not exist", git_dir.display()));
        }

        Ok(Self(path))
    }

    /// Get the repo path.
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Create a git command with the given args.
    fn get_git_command<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new("git");
        cmd.arg("-C");
        cmd.arg(self.path());
        cmd.args(args);
        cmd
    }

    /// Get the subject of the commit message for the given commit.
    pub fn get_commit_message_body(&self, commit_sha: &str) -> Result<String> {
        let cmd = self.get_git_command([
            "log",
            "-1",
            // Only get the body of the commit message.
            "--format=format:%b",
            commit_sha,
        ]);
        let output = get_cmd_stdout_utf8(cmd)?;
        Ok(output)
    }

    /// Get the subject of the commit message for the given commit.
    pub fn get_commit_message_subject(&self, commit_sha: &str) -> Result<String> {
        let cmd = self.get_git_command([
            "log",
            "-1",
            // Only get the subject of the commit message.
            "--format=format:%s",
            commit_sha,
        ]);
        let output = get_cmd_stdout_utf8(cmd)?;
        Ok(output)
    }

    /// Fetch git tags from the remote.
    pub fn fetch_git_tags(&self) -> Result<()> {
        let cmd = self.get_git_command(["fetch", "--tags"]);
        run_cmd(cmd)?;
        Ok(())
    }

    /// Check if a git tag exists locally.
    ///
    /// All git tags were fetched at the start of auto-release, so checking locally
    /// is sufficient.
    pub fn does_git_tag_exist(&self, tag: &str) -> Result<bool> {
        let cmd = self.get_git_command(["tag", "--list", tag]);
        let output = get_cmd_stdout_utf8(cmd)?;

        Ok(output.lines().any(|line| line == tag))
    }

    /// Create a git tag locally and push it.
    pub fn make_and_push_git_tag(&self, tag: &str, commit_sha: &str) -> Result<()> {
        // Create the tag.
        let cmd = self.get_git_command(["tag", tag, commit_sha]);
        run_cmd(cmd)?;

        // Push it.
        let cmd = self.get_git_command(["push", "--tags"]);
        run_cmd(cmd)?;

        Ok(())
    }
}
