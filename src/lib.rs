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
//! Example code for an [xtask](https://github.com/matklad/cargo-xtask) job:
//!
//! ```
//! use anyhow::Result;
//! use release_utils::git::*;
//! use release_utils::release::*;
//! use release_utils::Package;
//!
//! /// Entry point for the auto-release process. This is intended to be run
//! /// from a Github Actions workflow.
//! pub fn auto_release() -> Result<()> {
//!     let commit_sha = get_commit_sha()?;
//!     let commit_message_subject = get_commit_message_subject(&commit_sha)?;
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
//! Example Github Actions workflow:
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

#![warn(missing_docs)]

mod package;

pub mod cmd;
pub mod git;
pub mod release;

pub use package::Package;
