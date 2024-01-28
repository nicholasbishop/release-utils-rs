// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::github::{CreateRelease, Gh};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

const WRITE_ARGS_TEMPLATE: &str = r#"#!/bin/sh
echo "${@}" > OUT_PATH
exit EXIT_VAL
"#;

struct Script {
    exe_path: PathBuf,
    output_path: PathBuf,
}

impl Script {
    fn create(tmp_dir: &TempDir, exit_val: u32) -> Self {
        let exe_path = tmp_dir.path().join("write_args.sh");
        let output_path = tmp_dir.path().join("args");

        let content = WRITE_ARGS_TEMPLATE
            .replace("OUT_PATH", output_path.to_str().unwrap())
            .replace("EXIT_VAL", &exit_val.to_string());

        fs::write(&exe_path, content).unwrap();

        // TODO: this should be equivalent to:
        //
        // `fs::set_permissions(&exe_path, PermissionsExt::from_mode(0o700)).unwrap();`
        //
        // But for some reason, the executable sometimes fails to launch
        // with ETXTBSY when I do that. I have no good reason to suspect
        // that using a chmod subprocess here is actually fixing it;
        // just as likely that it's changing the timing enough to
        // accidentally avoid the problem.
        //
        // My best guess is that one of the `std::fs` calls is opening
        // the file without O_CLOEXEC, and the test runner is forking a
        // process with the file still open in write mode, but not sure
        // if that's correct.
        //
        // Since this is just test code, we can let this hack stand for now.
        let mut cmd = std::process::Command::new("chmod");
        cmd.arg("700");
        cmd.arg(&exe_path);
        release_utils::cmd::run_cmd(cmd).unwrap();

        Self {
            exe_path,
            output_path,
        }
    }
}

#[test]
fn test_gh_new_default() {
    assert_eq!(Gh::new(), Gh::default());
}

#[test]
fn test_gh_does_release_exist() {
    let tmp_dir = TempDir::new().unwrap();

    let script = Script::create(&tmp_dir, 0);
    let gh = Gh::with_exe(script.exe_path);
    assert_eq!(gh.does_release_exist("some-tag").unwrap(), true);
    assert_eq!(
        fs::read_to_string(script.output_path).unwrap(),
        "release view some-tag\n"
    );

    let script = Script::create(&tmp_dir, 1);
    let gh = Gh::with_exe(script.exe_path);
    assert_eq!(gh.does_release_exist("some-tag").unwrap(), false);

    let script = Script::create(&tmp_dir, 2);
    let gh = Gh::with_exe(script.exe_path);
    assert!(gh.does_release_exist("some-tag").is_err());
}

#[test]
fn test_gh_create_release() {
    let tmp_dir = TempDir::new().unwrap();

    let script = Script::create(&tmp_dir, 0);
    let gh = Gh::with_exe(script.exe_path);
    gh.create_release(CreateRelease {
        tag: "some-tag".to_string(),
        title: Some("title".to_string()),
        notes: Some("l1\nl2".to_string()),
        files: vec![PathBuf::from("f1"), PathBuf::from("f2")],
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(script.output_path).unwrap(),
        "release create --verify-tag --title title --notes l1\nl2 some-tag f1 f2\n"
    );
}
