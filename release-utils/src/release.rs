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
    get_github_sha, CrateRegistry, GetCrateVersionsError, Package, Repo,
};
use anyhow::Result;
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
    let commit_sha = get_github_sha()?;

    let repo = Repo::open()?;
    repo.fetch_git_tags()?;

    for package in packages {
        auto_release_package(&repo, package, &commit_sha)?;
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
    commit_sha: &str,
) -> Result<()> {
    let local_version = package.get_local_version()?;
    println!("local version of {} is {local_version}", package.name());

    // Create the crates.io release if it doesn't exist.
    // TODO: unwrap
    if does_crates_io_release_exist(package, &local_version).unwrap() {
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
