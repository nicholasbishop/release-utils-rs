// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cmd::{
    format_cmd, get_cmd_stdout_utf8, wait_for_child, RunCommandError,
};
use std::fmt::{self, Display, Formatter};
use std::io::Read;
use std::process::{Command, Stdio};

/// Error returned by [`Cargo::get_crate_versions`].
#[derive(Debug)]
pub enum GetCrateVersionsError {
    /// The crate has not yet been published.
    NotPublished,

    /// An internal error occurred.
    Internal {
        /// Description of the internal error.
        msg: String,

        /// Optional underlying error.
        cause: Option<Box<dyn std::error::Error + 'static>>,
    },
}

impl Display for GetCrateVersionsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to get crate versions: ")?;
        match self {
            Self::NotPublished => write!(f, "crate has not yet been published"),
            Self::Internal { msg, .. } => {
                write!(f, "{msg}")
            }
        }
    }
}

impl std::error::Error for GetCrateVersionsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NotPublished => None,
            Self::Internal { cause, .. } => cause.as_ref().map(|err| &**err),
        }
    }
}

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
        let mut cmd = Command::new("curl");
        cmd.args(["--silent"]);
        // Write the HTTP status code to stderr.
        cmd.args(["--write-out", "%{stderr}%{http_code}"]);
        cmd.arg(self.get_crate_url(crate_name));
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::piped());
        let curl_cmd_str = format_cmd(&cmd);
        let mut curl_proc =
            cmd.spawn().map_err(|err| GetCrateVersionsError::Internal {
                msg: "failed to launch curl".to_string(),
                cause: Some(Box::new(RunCommandError::Launch {
                    cmd: curl_cmd_str.clone(),
                    err,
                })),
            })?;

        // OK to unwrap, we know stderr and stdout are set.
        let mut curl_stderr_pipe = curl_proc.stderr.take().unwrap();
        let curl_stdout_pipe = curl_proc.stdout.take().unwrap();

        let versions_result = parse_versions_from_crate_json(curl_stdout_pipe);

        wait_for_child(curl_proc, curl_cmd_str).map_err(|err| {
            GetCrateVersionsError::Internal {
                msg: "curl failed".to_string(),
                cause: Some(Box::new(err)),
            }
        })?;

        let mut stderr_bytes = Vec::new();
        // TODO: unwraps
        curl_stderr_pipe.read_to_end(&mut stderr_bytes).unwrap();

        let stderr = String::from_utf8(stderr_bytes).unwrap();
        dbg!(&stderr);

        let code: i32 = stderr.trim().parse().map_err(|_| {
            GetCrateVersionsError::Internal {
                msg: format!("invalid HTTP code: {stderr:?}"),
                cause: None,
            }
        })?;
        if code == 404 {
            return Err(GetCrateVersionsError::NotPublished);
        }
        if code != 200 {
            return Err(GetCrateVersionsError::Internal {
                msg: format!("invalid HTTP code: {code}"),
                cause: None,
            });
        }

        versions_result.map_err(|err| GetCrateVersionsError::Internal {
            msg: "jq failed".to_string(),
            cause: Some(Box::new(err)),
        })
    }
}

fn parse_versions_from_crate_json(
    input: impl Into<Stdio>,
) -> Result<Vec<String>, RunCommandError> {
    let mut cmd = Command::new("jq");
    // Remove quotes.
    cmd.arg("--raw-output");
    // Select the version field.
    cmd.arg(".vers");
    cmd.stdin(input);
    let output = get_cmd_stdout_utf8(cmd)?;

    Ok(output.lines().map(|l| l.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TempDir;
    use std::fs::{self, File};

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
        let file = File::open(path).unwrap();
        let versions = parse_versions_from_crate_json(file).unwrap();
        assert_eq!(versions, ["0.2.4", "0.3.0", "0.4.0", "0.4.1"]);
    }
}
