// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::{CrateRegistry, GetCrateVersionsError};

#[test]
fn test_get_crate_versions() {
    let cargo = CrateRegistry::new();
    let versions = cargo.get_crate_versions("release-utils").unwrap();
    assert!(versions.contains(&"0.2.4".to_string()));
    assert!(versions.contains(&"0.3.0".to_string()));
    assert!(versions.contains(&"0.4.0".to_string()));
    assert!(versions.contains(&"0.4.1".to_string()));

    let cargo = CrateRegistry::new();
    assert!(matches!(
        cargo
            .get_crate_versions("does-not-exist-92452")
            .unwrap_err(),
        GetCrateVersionsError::NotPublished
    ));
}
