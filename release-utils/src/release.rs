// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for automatically releasing Rust code.

use crate::cmd::{run_cmd, RunCommandError};
use crate::{get_github_sha, GetLocalVersionError, Package, Repo, VarError};
use anyhow::Result;
use crates_index::SparseIndex;
use std::fmt::{self, Display, Formatter};
use std::process::Command;

/// Error returned by [`release_packages`].
#[derive(Debug)]
pub enum ReleasePackagesError {
    /// Environment error.
    Env(VarError),

    /// A git error occurred.
    Git(Box<dyn std::error::Error + Send + Sync + 'static>),

    /// A crate registry error occurred.
    CrateRegistry(crates_index::Error),

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
            Self::CrateRegistry(_) => write!(f, "crate registry error"),
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
            Self::CrateRegistry(err) => Some(err),
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

    let mut index = SparseIndex::new_cargo_default()
        .map_err(ReleasePackagesError::CrateRegistry)?;

    for package in packages {
        auto_release_package(&repo, package, &mut index, &commit_sha).map_err(
            |err| ReleasePackagesError::Package {
                package: package.name().to_string(),
                cause: err,
            },
        )?;
    }

    Ok(())
}

/// Error returned by [`auto_release_package`].
#[derive(Debug)]
pub enum ReleasePackageError {
    /// Failed to get the local version.
    LocalVersion(GetLocalVersionError),

    /// Failed to get the published versions of the crate.
    RemoteVersions(anyhow::Error),

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
            // TODO: remove anyhow.
            Self::RemoteVersions(_) => None,
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
    index: &mut SparseIndex,
    commit_sha: &str,
) -> Result<(), ReleasePackageError> {
    let local_version = package
        .get_local_version()
        .map_err(ReleasePackageError::LocalVersion)?;
    println!("local version of {} is {local_version}", package.name());

    // Create the crates.io release if it doesn't exist.
    if does_crates_io_release_exist(package, &local_version, index)
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

/// Returned by [`update_index`] to indicate whether a crate exists on
/// crates.io.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[must_use]
pub struct RemoteCrateExists(pub bool);

/// Update the local crates.io cache.
///
/// Based on <https://github.com/frewsxcv/rust-crates-index/blob/HEAD/examples/sparse_http_ureq.rs>
pub fn update_index(
    index: &mut SparseIndex,
    package: &Package,
) -> Result<RemoteCrateExists> {
    let crate_name = package.name();

    println!("fetching updates for {}", package.name());
    let request: ureq::Request = index.make_cache_request(crate_name)?.into();
    match request.call() {
        Ok(response) => {
            index.parse_cache_response(crate_name, response.into(), true)?;
            Ok(RemoteCrateExists(true))
        }
        // Handle the case where the package does not yet have any
        // releases.
        Err(ureq::Error::Status(404, _)) => {
            println!("packages {} does not exist yet", package.name());
            Ok(RemoteCrateExists(false))
        }
        Err(err) => Err(err.into()),
    }
}

/// Check if a new release of `package` should be published.
pub fn does_crates_io_release_exist(
    package: &Package,
    local_version: &str,
    index: &mut SparseIndex,
) -> Result<bool> {
    let remote_versions = get_remote_package_versions(package, index)?;
    if remote_versions.contains(&local_version.to_string()) {
        return Ok(true);
    }

    Ok(false)
}

/// Get all remote versions of `package`.
pub fn get_remote_package_versions(
    package: &Package,
    index: &mut SparseIndex,
) -> Result<Vec<String>> {
    // The local cache may be out of date, fetch updates from the remote.
    let exists = update_index(index, package)?;

    // If the crate hasn't been published yet, return an empty list of versions.
    if !exists.0 {
        return Ok(Vec::new());
    }

    let cr = index.crate_from_cache(package.name())?;

    Ok(cr
        .versions()
        .iter()
        .map(|v| v.version().to_string())
        .collect())
}

/// Publish `package` to crates.io.
pub fn publish_package(package: &Package) -> Result<(), RunCommandError> {
    let mut cmd = Command::new("cargo");
    cmd.args(["publish", "--package", package.name()]);
    run_cmd(cmd)
}
