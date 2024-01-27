// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use std::env;
use std::path::{Path, PathBuf};

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

    /// Use the `cargo_metadata` crate to get the local version of a package
    /// in the workspace.
    pub fn get_local_version(&self) -> Result<String> {
        let mut cmd = MetadataCommand::new();
        cmd.manifest_path(self.workspace.join("Cargo.toml"));
        // Ignore deps, we only need local packages.
        cmd.no_deps();
        let local_metadata = cmd.exec()?;

        let metadata = local_metadata
            .packages
            .iter()
            .find(|pm| pm.name == self.name)
            .context(format!("failed to find {} in local metadata", self.name))?;
        Ok(metadata.version.to_string())
    }
}
