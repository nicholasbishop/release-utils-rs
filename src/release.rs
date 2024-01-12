// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for automatically releasing Rust code.

use crate::cmd::run_cmd;
use crate::git::{does_git_tag_exist, fetch_git_tags, make_and_push_git_tag};
use crate::package::Package;
use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use crates_index::SparseIndex;
use std::env;
use std::process::Command;

/// Entry point for the auto-release process. This is intended to be run from a
/// Github Actions workflow, see `.github/workflows/release.yml`.
pub fn release_packages(packages: &[Package]) -> Result<()> {
    let commit_sha = get_commit_sha()?;

    fetch_git_tags()?;

    let local_metadata = get_local_package_metadata()?;
    let mut index = SparseIndex::new_cargo_default()?;

    for package in packages {
        auto_release_package(package, &local_metadata, &mut index, &commit_sha)?;
    }

    Ok(())
}

/// Release a single package, if needed.
///
/// This publishes to crates.io if the corresponding version does not already
/// exist there, and also pushes a new git tag if one doesn't exist yet.
pub fn auto_release_package(
    package: &Package,
    local_metadata: &Metadata,
    index: &mut SparseIndex,
    commit_sha: &str,
) -> Result<()> {
    let local_version = get_local_package_version(package, local_metadata)?;
    println!("local version of {} is {local_version}", package.name());

    // Create the remote git tag if it doesn't exist.
    let tag = package.get_git_tag_name(&local_version);
    if does_git_tag_exist(&tag)? {
        println!("git tag {tag} already exists");
    } else {
        make_and_push_git_tag(&tag, commit_sha)?;
    }

    // Create the crates.io release if it doesn't exist.
    if does_crates_io_release_exist(package, &local_version, index)? {
        println!(
            "{}-{local_version} has already been published",
            package.name()
        );
    } else {
        publish_package(package)?;
    }

    Ok(())
}

/// Get the commit to operate on from the `GITHUB_SHA` env var. When running in
/// Github Actions, this will be set to the SHA of the merge commit that was
/// pushed to the branch.
fn get_commit_sha() -> Result<String> {
    let commit_var_name = "GITHUB_SHA";
    env::var(commit_var_name).context(format!("failed to get env var {commit_var_name}"))
}

/// Use the `cargo_metadata` crate to get local info about packages in the
/// workspace.
fn get_local_package_metadata() -> Result<Metadata> {
    let mut cmd = MetadataCommand::new();
    // Ignore deps, we only need local packages.
    cmd.no_deps();
    Ok(cmd.exec()?)
}

/// Update the local crates.io cache.
///
/// Based on <https://github.com/frewsxcv/rust-crates-index/blob/HEAD/examples/sparse_http_ureq.rs>
fn update_index(index: &mut SparseIndex, package: &Package) -> Result<()> {
    let crate_name = package.name();

    println!("fetching updates for {}", package.name());
    let request: ureq::Request = index.make_cache_request(crate_name).unwrap().into();
    let response = request.call()?;

    index.parse_cache_response(crate_name, response.into(), true)?;

    Ok(())
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

/// Get the local version of `package`.
pub fn get_local_package_version(package: &Package, local_metadata: &Metadata) -> Result<String> {
    let metadata = local_metadata
        .packages
        .iter()
        .find(|pm| pm.name == package.name())
        .context(format!(
            "failed to find {} in local metadata",
            package.name()
        ))?;
    Ok(metadata.version.to_string())
}

/// Get all remote versions of `package`.
pub fn get_remote_package_versions(
    package: &Package,
    index: &mut SparseIndex,
) -> Result<Vec<String>> {
    // The local cache may be out of date, fetch updates from the remote.
    update_index(index, package)?;

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
    run_cmd(cmd)
}
