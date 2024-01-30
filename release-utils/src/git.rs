// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for running `git` commands.

use crate::cmd::{get_cmd_stdout_utf8, run_cmd, RunCommandError};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, io};

/// Error returned by [`Repo::open`] and [`Repo::open_path`].
#[derive(Debug)]
pub enum RepoOpenError {
    /// Failed to get current directory.
    CurrentDir(io::Error),

    /// The directory does not have a `.git` subdirectory.
    GitDirMissing(PathBuf),
}

impl Display for RepoOpenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrentDir(err) => write!(f, "failed to get current dir: {err}"),
            Self::GitDirMissing(path) => write!(f, "{} does not exist", path.display()),
        }
    }
}

impl std::error::Error for RepoOpenError {}

/// Git repo.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Repo(PathBuf);

impl Repo {
    /// Get a `Repo` for the current directory.
    ///
    /// This will fail if the current directory does not contain a
    /// `.git` subdirectory.
    pub fn open() -> Result<Self, RepoOpenError> {
        let path = env::current_dir().map_err(RepoOpenError::CurrentDir)?;
        Self::open_path(path)
    }

    /// Get a `Repo` for the given path.
    ///
    /// This will fail if the `path` does not contain a `.git`
    /// subdirectory.
    pub fn open_path<P>(path: P) -> Result<Self, RepoOpenError>
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
            return Err(RepoOpenError::GitDirMissing(git_dir));
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
    pub fn get_commit_message_body(&self, commit_sha: &str) -> Result<String, RunCommandError> {
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
    pub fn get_commit_message_subject(&self, commit_sha: &str) -> Result<String, RunCommandError> {
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
    pub fn fetch_git_tags(&self) -> Result<(), RunCommandError> {
        let cmd = self.get_git_command(["fetch", "--tags"]);
        run_cmd(cmd)?;
        Ok(())
    }

    /// Check if a git tag exists locally.
    ///
    /// All git tags were fetched at the start of auto-release, so checking locally
    /// is sufficient.
    pub fn does_git_tag_exist(&self, tag: &str) -> Result<bool, RunCommandError> {
        let cmd = self.get_git_command(["tag", "--list", tag]);
        let output = get_cmd_stdout_utf8(cmd)?;

        Ok(output.lines().any(|line| line == tag))
    }

    /// Create a git tag locally and push it.
    pub fn make_and_push_git_tag(
        &self,
        tag: &str,
        commit_sha: &str,
    ) -> Result<(), RunCommandError> {
        // Create the tag.
        let cmd = self.get_git_command(["tag", tag, commit_sha]);
        run_cmd(cmd)?;

        // Push it.
        let cmd = self.get_git_command(["push", "--tags"]);
        run_cmd(cmd)?;

        Ok(())
    }
}
