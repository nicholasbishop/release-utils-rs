// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// A package in the workspace.
pub struct Package(String);

impl Package {
    /// Create a `Package` with the given name.
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    /// Get the package's name.
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Format a package version as a git tag.
    pub fn get_git_tag_name(&self, local_version: &str) -> String {
        format!("{}-v{}", self.name(), local_version)
    }
}
