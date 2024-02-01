// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;
use std::fmt::{self, Display, Formatter};

/// Error getting an environment variable
///
/// Wrapper around [`std::env::VarError`] that adds the name of the
/// variable. This provides a more useful error message.
#[derive(Debug, PartialEq)]
pub struct VarError {
    /// Name of the environment variable.
    pub name: String,

    /// The underlying error.
    pub err: env::VarError,
}

impl Display for VarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, r#"failed to read "{}" from the env"#, self.name)
    }
}

impl std::error::Error for VarError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

/// Get the commit to operate on from the `GITHUB_SHA` env var. When
/// running in Github Actions, this will be set to the SHA of the commit
/// that triggered the workflow.
///
/// See Github Actions' [Variables] documentation for details.
///
/// [Variables]: https://docs.github.com/en/actions/learn-github-actions/variables
pub fn get_github_sha() -> Result<String, VarError> {
    let commit_var_name = "GITHUB_SHA";
    env::var(commit_var_name).map_err(|err| VarError {
        name: commit_var_name.to_owned(),
        err,
    })
}
