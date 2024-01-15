// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for running child processes.

use std::fmt::{self, Display, Formatter};
use std::io;
use std::process::{Command, ExitStatus};

/// Error returned when running a child process fails.
#[derive(Debug)]
pub enum RunCommandError {
    /// Failed to launch command. May indicate the program is not installed.
    Launch {
        /// Stringified form of the command that failed.
        cmd: String,
        /// Underlying error.
        err: io::Error,
    },

    /// The command exited with a non-zero code, or was terminated by a
    /// signal.
    NonZeroExit {
        /// Stringified form of the command that failed.
        cmd: String,
        /// Exit status.
        status: ExitStatus,
    },
}

impl Display for RunCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::Launch { cmd, err } => {
                write!(f, "failed to launch command \"{cmd}\": {err}")
            }
            Self::NonZeroExit { cmd, status } => {
                write!(f, "command \"{cmd}\" failed with {status}")
            }
        }
    }
}

impl std::error::Error for RunCommandError {}

/// Format a command in a way suitable for logging.
pub fn format_cmd(cmd: &Command) -> String {
    format!("{cmd:?}").replace('"', "")
}

/// Log a command and run it.
///
/// Returns an error if the process fails to launch or if the exit code
/// is non-zero.
pub fn run_cmd(mut cmd: Command) -> Result<(), RunCommandError> {
    let cmd_str = format_cmd(&cmd);
    println!("Running: {}", cmd_str);
    let status = cmd.status().map_err(|err| RunCommandError::Launch {
        cmd: cmd_str.clone(),
        err,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(RunCommandError::NonZeroExit {
            cmd: cmd_str,
            status,
        })
    }
}

/// Log a command, run it, and get its output.
///
/// Returns an error if the process fails to launch or if the exit code
/// is non-zero.
pub fn get_cmd_stdout(mut cmd: Command) -> Result<Vec<u8>, RunCommandError> {
    let cmd_str = format_cmd(&cmd);
    println!("Running: {}", cmd_str);
    let output = cmd.output().map_err(|err| RunCommandError::Launch {
        cmd: cmd_str.clone(),
        err,
    })?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(RunCommandError::NonZeroExit {
            cmd: cmd_str,
            status: output.status,
        })
    }
}