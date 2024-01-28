// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::TempDir;
use std::fs;

#[test]
fn test_tmp_dir() {
    let path;

    {
        let tmp_dir = TempDir::new().unwrap();
        path = tmp_dir.path().to_path_buf();
        assert!(path.exists());

        fs::write(path.join("file.txt"), "hello\n").unwrap();
    }

    assert!(!path.exists());
}
