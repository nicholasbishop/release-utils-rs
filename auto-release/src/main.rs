// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, required = true)]
    package: Vec<String>,

    #[arg(long)]
    execute: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.execute {
        todo!();
    } else {
        println!("--execute not passed; stopping");
    }
}
