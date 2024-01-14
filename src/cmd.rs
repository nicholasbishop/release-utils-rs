// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for running child processes.

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Format a command in a way suitable for logging.
pub fn format_cmd(cmd: &Command) -> String {
    format!("{cmd:?}").replace('"', "")
}

/// Log a command and run it.
pub fn run_cmd(mut cmd: Command) -> Result<()> {
    println!("Running: {}", format_cmd(&cmd));
    let status = cmd.status().context("failed to launch process")?;
    if status.success() {
        Ok(())
    } else {
        bail!("command failed: {status}");
    }
}

/// Log a command, run it, and get its output.
pub fn get_cmd_stdout(mut cmd: Command) -> Result<Vec<u8>> {
    println!("Running: {}", format_cmd(&cmd));
    let output = cmd.output().context("failed to launch process")?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        bail!("command failed: {}", output.status);
    }
}
