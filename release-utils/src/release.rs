// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for automatically releasing Rust code.

use crate::cmd::{run_cmd, RunCommandError};
use crate::{
    get_github_sha, CrateRegistry, GetCrateVersionsError, GetLocalVersionError,
    Package, Repo, VarError,
};
use std::fmt::{self, Display, Formatter};
use std::process::Command;

/// Error returned by [`release_packages`].
#[derive(Debug)]
pub enum ReleasePackagesError {
    /// Environment error.
    Env(VarError),

    /// A git error occurred.
    Git(Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Failed to release a package.
    Package {
        /// Name of the package.
        package: String,
        /// Underlying error.
        cause: ReleasePackageError,
    },
}

impl Display for ReleasePackagesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env(_) => write!(f, "environment error"),
            Self::Git(_) => write!(f, "git error"),
            Self::Package { package, .. } => {
                write!(f, "failed to release package {package}")
            }
        }
    }
}

impl std::error::Error for ReleasePackagesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Env(err) => Some(err),
            Self::Git(err) => Some(&**err),
            Self::Package { cause, .. } => Some(cause),
        }
    }
}

/// Release each package in `packages`, if needed.
///
/// For each package, this will create a remote git tag (if it doesn't
/// already exist) and a crates.io release (if it doesn't already
/// exist).
///
/// Note that when releasing to crates.io, the order of `packages` may
/// be significant if the packages depend on one another.
pub fn release_packages(
    packages: &[Package],
) -> Result<(), ReleasePackagesError> {
    let commit_sha = get_github_sha().map_err(ReleasePackagesError::Env)?;

    let repo =
        Repo::open().map_err(|err| ReleasePackagesError::Git(Box::new(err)))?;
    repo.fetch_git_tags()
        .map_err(|err| ReleasePackagesError::Git(Box::new(err)))?;

    for package in packages {
        auto_release_package(&repo, package, &commit_sha).map_err(|err| {
            ReleasePackagesError::Package {
                package: package.name().to_string(),
                cause: err,
            }
        })?;
    }

    Ok(())
}

/// Error returned by [`auto_release_package`].
#[derive(Debug)]
pub enum ReleasePackageError {
    /// Failed to get the local version.
    LocalVersion(GetLocalVersionError),

    /// Failed to get the published versions of the crate.
    RemoteVersions(GetCrateVersionsError),

    /// Failed to publish the crate.
    Publish(RunCommandError),

    /// Failed to create or push the git tag.
    Git(RunCommandError),
}

impl Display for ReleasePackageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalVersion(_) => {
                write!(f, "failed to get local package version")
            }
            Self::RemoteVersions(_) => {
                write!(f, "failed to get the published package versions")
            }
            Self::Publish(_) => write!(f, "failed to publish the crate"),
            Self::Git(_) => write!(f, "git error"),
        }
    }
}

impl std::error::Error for ReleasePackageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LocalVersion(err) => Some(err),
            Self::RemoteVersions(err) => Some(err),
            Self::Publish(err) => Some(err),
            Self::Git(err) => Some(err),
        }
    }
}

/// Release a single package, if needed.
///
/// This publishes to crates.io if the corresponding version does not already
/// exist there, and also pushes a new git tag if one doesn't exist yet.
pub fn auto_release_package(
    repo: &Repo,
    package: &Package,
    commit_sha: &str,
) -> Result<(), ReleasePackageError> {
    let local_version = package
        .get_local_version()
        .map_err(ReleasePackageError::LocalVersion)?;
    println!("local version of {} is {local_version}", package.name());

    // Create the crates.io release if it doesn't exist.
    if does_crates_io_release_exist(package, &local_version)
        .map_err(ReleasePackageError::RemoteVersions)?
    {
        println!(
            "{}-{local_version} has already been published",
            package.name()
        );
    } else {
        publish_package(package).map_err(ReleasePackageError::Publish)?;
    }

    // Create the remote git tag if it doesn't exist.
    let tag = package.get_git_tag_name(&local_version);
    if repo
        .does_git_tag_exist(&tag)
        .map_err(ReleasePackageError::Git)?
    {
        println!("git tag {tag} already exists");
    } else {
        repo.make_and_push_git_tag(&tag, commit_sha)
            .map_err(ReleasePackageError::Git)?;
    }

    Ok(())
}

/// Check if a new release of `package` should be published.
pub fn does_crates_io_release_exist(
    package: &Package,
    local_version: &str,
) -> Result<bool, GetCrateVersionsError> {
    let cargo = CrateRegistry::new();
    let remote_versions = cargo.get_crate_versions(package.name())?;

    if remote_versions.contains(&local_version.to_string()) {
        return Ok(true);
    }

    Ok(false)
}

/// Publish `package` to crates.io.
pub fn publish_package(package: &Package) -> Result<(), RunCommandError> {
    let mut cmd = Command::new("cargo");
    cmd.args(["publish", "--package", package.name()]);
    run_cmd(cmd)
}
