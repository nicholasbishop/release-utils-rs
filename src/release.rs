// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for automatically releasing Rust code.

use crate::cmd::run_cmd;
use crate::{Package, Repo};
use anyhow::{Context, Result};
use crates_index::SparseIndex;
use std::env;
use std::process::Command;

/// Release each package in `packages`, if needed.
///
/// For each package, this will create a remote git tag (if it doesn't
/// already exist) and a crates.io release (if it doesn't already
/// exist).
///
/// Note that when releasing to crates.io, the order of `packages` may
/// be significant if the packages depend on one another.
pub fn release_packages(packages: &[Package]) -> Result<()> {
    let commit_sha = get_commit_sha()?;

    let repo = Repo::open()?;
    repo.fetch_git_tags()?;

    let mut index = SparseIndex::new_cargo_default()?;

    for package in packages {
        auto_release_package(&repo, package, &mut index, &commit_sha)?;
    }

    Ok(())
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
) -> Result<()> {
    let local_version = package.get_local_version()?;
    println!("local version of {} is {local_version}", package.name());

    // Create the crates.io release if it doesn't exist.
    if does_crates_io_release_exist(package, &local_version, index)? {
        println!(
            "{}-{local_version} has already been published",
            package.name()
        );
    } else {
        publish_package(package)?;
    }

    // Create the remote git tag if it doesn't exist.
    let tag = package.get_git_tag_name(&local_version);
    if repo.does_git_tag_exist(&tag)? {
        println!("git tag {tag} already exists");
    } else {
        repo.make_and_push_git_tag(&tag, commit_sha)?;
    }

    Ok(())
}

/// Get the commit to operate on from the `GITHUB_SHA` env var. When running in
/// Github Actions, this will be set to the SHA of the merge commit that was
/// pushed to the branch.
pub fn get_commit_sha() -> Result<String> {
    let commit_var_name = "GITHUB_SHA";
    env::var(commit_var_name).context(format!("failed to get env var {commit_var_name}"))
}

/// Returned by [`update_index`] to indicate whether a crate exists on
/// crates.io.
#[must_use]
pub struct RemoteCrateExists(pub bool);

/// Update the local crates.io cache.
///
/// Based on <https://github.com/frewsxcv/rust-crates-index/blob/HEAD/examples/sparse_http_ureq.rs>
pub fn update_index(index: &mut SparseIndex, package: &Package) -> Result<RemoteCrateExists> {
    let crate_name = package.name();

    println!("fetching updates for {}", package.name());
    let request: ureq::Request = index.make_cache_request(crate_name).unwrap().into();
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
pub fn publish_package(package: &Package) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["publish", "--package", package.name()]);
    run_cmd(cmd)?;
    Ok(())
}
