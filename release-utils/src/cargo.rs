// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cmd::{get_cmd_stdout_utf8, RunCommandError};
use crate::TempDir;
use std::fmt::{self, Display, Formatter};
use std::path::Path;
use std::process::Command;

/// Error returned by [`Cargo::get_crate_versions`].
#[derive(Debug)]
pub enum GetCrateVersionsError {
    /// The crate has not yet been published.
    NotPublished,

    /// `curl` failed.
    Curl(RunCommandError),

    /// The HTTP code is not a valid number.
    InvalidHttpCode(String),

    /// The HTTP code was neither 200 nor 404.
    UnexpectedHttpCode(u32),

    /// `jq` failed.
    Jq(RunCommandError),

    /// Failed to create a temporary directory.
    TempDir(RunCommandError),
}

impl Display for GetCrateVersionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotPublished => write!(f, "crate has not been published yet"),
            Self::Curl(err) => write!(f, "curl failed: {err}"),
            Self::InvalidHttpCode(_) => write!(f, "invalid HTTP code"),
            Self::UnexpectedHttpCode(code) => {
                write!(f, "unexpected HTTP code: {code}")
            }
            Self::Jq(err) => write!(f, "jq failed: {err}"),
            Self::TempDir(err) => {
                write!(f, "failed to create temporary directory: {err}")
            }
        }
    }
}

impl std::error::Error for GetCrateVersionsError {}

/// Access a crate registry.
pub struct CrateRegistry {
    /// Base URL of the sparse registry.
    pub registry_url: String,
}

impl CrateRegistry {
    /// URL for the crates.io registry.
    pub const DEFAULT_REGISTRY: &'static str = "https://index.crates.io";

    /// Create a new `CrateRegistry` with the default registry.
    pub fn new() -> Self {
        Self {
            registry_url: Self::DEFAULT_REGISTRY.to_string(),
        }
    }

    /// Get the URL of the crate in the registry.
    fn get_crate_url(&self, crate_name: &str) -> String {
        assert!(!crate_name.is_empty());

        let mut url = self.registry_url.clone();
        if !url.ends_with('/') {
            url.push('/');
        }

        // https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files
        if crate_name.len() == 1 {
            url.push_str(&format!("1/{crate_name}"));
        } else if crate_name.len() == 2 {
            url.push_str(&format!("2/{crate_name}"));
        } else if crate_name.len() == 3 {
            url.push_str(&format!("3/{}/{}", &crate_name[0..1], crate_name));
        } else {
            url.push_str(&format!(
                "{}/{}/{}",
                &crate_name[..2],
                &crate_name[2..4],
                crate_name
            ));
        }

        url
    }

    /// Get all published versions of a crate.
    ///
    /// If the crate has not yet been published,
    /// [`GetCrateVersionsError::NotPublished`] is returned.
    pub fn get_crate_versions(
        &self,
        crate_name: &str,
    ) -> Result<Vec<String>, GetCrateVersionsError> {
        let tmp_dir = TempDir::new().map_err(GetCrateVersionsError::TempDir)?;
        let output_path = tmp_dir.path().join("output.json");

        let mut cmd = Command::new("curl");
        cmd.args(["--silent", "--output"]);
        cmd.arg(&output_path);
        cmd.args(["--write-out", "%{http_code}"]);
        cmd.arg(self.get_crate_url(crate_name));

        let output =
            get_cmd_stdout_utf8(cmd).map_err(GetCrateVersionsError::Curl)?;

        let code = output
            .trim()
            .parse()
            .map_err(|_| GetCrateVersionsError::InvalidHttpCode(output))?;
        if code == 404 {
            return Err(GetCrateVersionsError::NotPublished);
        }
        if code != 200 {
            return Err(GetCrateVersionsError::UnexpectedHttpCode(code));
        }

        parse_versions_from_crate_json(&output_path)
    }
}

fn parse_versions_from_crate_json(
    input: &Path,
) -> Result<Vec<String>, GetCrateVersionsError> {
    let mut cmd = Command::new("jq");
    // Remove quotes.
    cmd.arg("--raw-output");
    // Select the version field.
    cmd.arg(".vers");
    cmd.arg(input);
    let output = get_cmd_stdout_utf8(cmd).map_err(GetCrateVersionsError::Jq)?;

    Ok(output.lines().map(|l| l.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_url() {
        let cargo = CrateRegistry::new();

        assert_eq!(cargo.get_crate_url("a"), "https://index.crates.io/1/a");

        assert_eq!(cargo.get_crate_url("aa"), "https://index.crates.io/2/aa");

        assert_eq!(
            cargo.get_crate_url("aaa"),
            "https://index.crates.io/3/a/aaa"
        );

        assert_eq!(
            cargo.get_crate_url("release-utils"),
            "https://index.crates.io/re/le/release-utils"
        );
    }

    #[test]
    fn test_jq() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().join("crate.json");
        fs::write(&path, r#"{"name":"release-utils","vers":"0.2.4","deps":[{"name":"anyhow","req":"^1.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"cargo_metadata","req":"^0.18.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"crates-index","req":"^2.3.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"ureq","req":"^2.8.0","features":["http-interop"],"optional":false,"default_features":true,"target":null,"kind":"normal"}],"cksum":"92959b131c3d34846e39fed70bd7504684df0c6937ae736860329bd67836922e","features":{},"yanked":false,"rust_version":"1.70"}
{"name":"release-utils","vers":"0.3.0","deps":[{"name":"anyhow","req":"^1.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"cargo_metadata","req":"^0.18.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"crates-index","req":"^2.3.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"tempfile","req":"^3.9.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"dev"},{"name":"ureq","req":"^2.8.0","features":["http-interop"],"optional":false,"default_features":true,"target":null,"kind":"normal"}],"cksum":"ce9721f93fd5cc4aa5cb82e9e550af437c55adfc49731984185e691442a932f9","features":{},"yanked":false,"rust_version":"1.70"}
{"name":"release-utils","vers":"0.4.0","deps":[{"name":"anyhow","req":"^1.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"cargo_metadata","req":"^0.18.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"crates-index","req":"^2.3.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"tempfile","req":"^3.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"dev"},{"name":"ureq","req":"^2.8.0","features":["http-interop"],"optional":false,"default_features":true,"target":null,"kind":"normal"}],"cksum":"0aa93a5aaaed004e0222a3207cf5ec5dc15a39baea0e412bebfb7aa7bb8fa14c","features":{},"yanked":false,"rust_version":"1.70"}
{"name":"release-utils","vers":"0.4.1","deps":[{"name":"anyhow","req":"^1.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"cargo_metadata","req":"^0.18.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"crates-index","req":"^2.3.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"normal"},{"name":"tempfile","req":"^3.0.0","features":[],"optional":false,"default_features":true,"target":null,"kind":"dev"},{"name":"ureq","req":"^2.8.0","features":["http-interop"],"optional":false,"default_features":true,"target":null,"kind":"normal"}],"cksum":"02922e087d9f1da9f783ca54f4621f1a156ffc3f8563d66c2d74b5d2d6363ccf","features":{},"yanked":false,"rust_version":"1.70"}
"#).unwrap();
        let versions = parse_versions_from_crate_json(&path).unwrap();
        assert_eq!(versions, ["0.2.4", "0.3.0", "0.4.0", "0.4.1"]);
    }
}
