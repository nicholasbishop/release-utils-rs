// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("expected one argument");
    }

    let action = &args[1];
    if action == "release" {
        // TODO
    } else {
        panic!("invalid action: {action}");
    }
}
