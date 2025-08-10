// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cmd::{
    RunCommandError, format_cmd, get_cmd_stdout_utf8, wait_for_child,
};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A package in the workspace.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Package {
    /// Path of the root workspace directory, or just the directory of
    /// the package in non-workspace projects.
    workspace: PathBuf,

    /// Name of the package.
    name: String,
}

impl Package {
    /// Create a `Package` with the given name.
    ///
    /// This uses the current directory as the the workspace path.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let workspace = env::current_dir().unwrap();
        Self::with_workspace(name, workspace)
    }

    /// Create a `Package` with the given name and workspace.
    ///
    /// The workspace directory should be the root of the workspace, or
    /// just the directory of the package in non-workspace projects.
    pub fn with_workspace<S, P>(name: S, workspace: P) -> Self
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        Self {
            workspace: workspace.into(),
            name: name.into(),
        }
    }

    /// Get the package's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the package's root workspace directory.
    pub fn workspace(&self) -> &Path {
        &self.workspace
    }

    /// Format a package version as a git tag.
    pub fn get_git_tag_name(&self, local_version: &str) -> String {
        format!("{}-v{}", self.name, local_version)
    }

    /// Use `cargo metadata` to get the local version of a package
    /// in the workspace.
    pub fn get_local_version(&self) -> Result<String, GetLocalVersionError> {
        // Spawn `cargo metadata`. The output goes to a new pipe, which
        // will be passed as the input to `jq`.
        let mut metadata_cmd = self.get_cargo_metadata_cmd();
        let metadata_cmd_str = format_cmd(&metadata_cmd);
        println!("Running: {metadata_cmd_str}");
        let mut metadata_proc =
            metadata_cmd.stdout(Stdio::piped()).spawn().map_err(|err| {
                GetLocalVersionError::Process(RunCommandError::Launch {
                    cmd: metadata_cmd_str.clone(),
                    err,
                })
            })?;

        // OK to unwrap, we know stdout is set.
        let pipe = metadata_proc.stdout.take().unwrap();

        let mut jq_cmd = Command::new("jq");
        jq_cmd.arg("--raw-output");
        jq_cmd.arg(format!(
            ".packages[] | select(.name == \"{}\") | .version",
            self.name
        ));
        jq_cmd.stdin(pipe);

        let mut output = get_cmd_stdout_utf8(jq_cmd)
            .map_err(GetLocalVersionError::Process)?;

        wait_for_child(metadata_proc, metadata_cmd_str)
            .map_err(GetLocalVersionError::Process)?;

        if output.is_empty() {
            Err(GetLocalVersionError::PackageNotFound(
                self.name().to_string(),
            ))
        } else {
            // Remove trailing newline.
            output.pop();
            Ok(output)
        }
    }

    fn get_cargo_metadata_cmd(&self) -> Command {
        let mut cmd = Command::new("cargo");
        cmd.arg("metadata");
        cmd.args(["--format-version", "1"]);
        cmd.arg("--manifest-path");
        cmd.arg(self.workspace.join("Cargo.toml"));
        // Ignore deps, we only need local packages.
        cmd.arg("--no-deps");
        cmd
    }
}

/// Error returned by [`Package::get_local_version`].
#[derive(Debug)]
pub enum GetLocalVersionError {
    /// A child process failed.
    Process(RunCommandError),

    /// Requested package not found in the metadata.
    PackageNotFound(String),
}

impl Display for GetLocalVersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Process(err) => {
                write!(f, "failed to get cargo metadata: {err}")
            }
            Self::PackageNotFound(pkg) => {
                write!(f, "package {pkg} not found in cargo metadata")
            }
        }
    }
}

impl std::error::Error for GetLocalVersionError {}
