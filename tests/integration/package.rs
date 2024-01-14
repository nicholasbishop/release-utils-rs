// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use release_utils::Package;

#[test]
fn test_package() {
    let pkg = Package::new("foo");
    assert_eq!(pkg.name(), "foo");
    assert_eq!(pkg.get_git_tag_name("1.2.3"), "foo-v1.2.3");
}
