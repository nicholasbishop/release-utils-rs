// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for automatically releasing Rust code.
//!
//! The intended usage is something like this (but not necessarily exactly
//! this):
//!
//! 1. All code changes needed for a release are made by a developer in a
//!    regular git commit. The commit includes bumping the version in
//!    `Cargo.toml`, and any updates to `Cargo.lock`, changelog files, etc.
//! 2. The commit message is prefixed with `release:` to mark the commit as
//!    a release trigger.
//! 3. The commit is reviewed and merged through the normal pull request
//!    process.
//! 4. Once merged, an automatic job sees the specially-marked commit and
//!    triggers any actions necessary to push the release. The building
//!    blocks for this automated part are what `release-utils-rs` provides.
//!
//! # Example code for a release job
//!
//! ```
//! use anyhow::Result;
//! use release_utils::release::*;
//! use release_utils::{get_github_sha, Package, Repo};
//!
//! /// Entry point for the auto-release process. This is intended to be run
//! /// from a Github Actions workflow.
//! fn auto_release() -> Result<()> {
//!     let commit_sha = get_github_sha()?;
//!     let repo = Repo::open()?;
//!     let commit_message_subject =
//!         repo.get_commit_message_subject(&commit_sha)?;
//!
//!     if !commit_message_subject.starts_with("release:") {
//!         println!("{commit_sha} does not contain the release trigger");
//!         return Ok(());
//!     }
//!
//!     release_packages(&[Package::new("foo"), Package::new("bar")])
//! }
//! ```
//!
//! # Example Github Actions workflow
//!
//! ```toml
//! on:
//!   push:
//!     branches:
//!       - main
//!
//! permissions:
//!   contents: write
//!
//! jobs:
//!   release:
//!     runs-on: ubuntu-latest
//!     steps:
//!       - uses: actions/checkout@v4
//!       - uses: Swatinem/rust-cache@v2
//!       - run: cargo xtask release
//!         env:
//!           CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
//! ```
//!
//! # Cargo registry token
//!
//! The above Github Actions workflow requires a secret token. Generate
//! the token in your crates.io [Account Settings]. The token scopes
//! must include `publish-update`. If the crate has never been published
//! before, `publish-new` is also required.
//!
//! To make the token available to the Github Actions workflow:
//! 1. Go to your repository's settings
//! 2. Click to `Secrets and variables` in the sidebar, then click `Actions`
//! 3. Under `Repository secrets`, click `New repository secret`.
//!
//! [Account Settings]: https://crates.io/settings/tokens

#![deny(unsafe_code)]
#![warn(missing_docs)]

mod cargo;
mod env;
mod git;
mod package;
mod tmp;

pub mod cmd;
pub mod github;
pub mod release;

pub use cargo::{CrateRegistry, GetCrateVersionsError};
pub use env::{get_github_sha, VarError};
pub use git::{Repo, RepoOpenError};
pub use package::{GetLocalVersionError, Package};
pub use tmp::TempDir;
