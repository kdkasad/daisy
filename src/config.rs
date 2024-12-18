/*
 * src/config.rs - Daisy - A ridiculous SSH daisy chain
 * Copyright (C) 2024  Kian Kasad <kian@kasad.com>
 * SPDX-License-Identifier: GPL-3.0-or-later
 *
 * This file is part of Daisy.
 *
 * Daisy is free software: you can redistribute it and/or modify it under the
 * terms of the GNU General Public License as published by the Free Software
 * Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * Daisy is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
 * A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Daisy. If not, see <https://www.gnu.org/licenses/>.
 */

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Daisy configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DaisyConfig {
    /// Index of this link within the chain. 0 is the sender.
    #[serde(default)]
    pub link_index: usize,

    /// The command to run on the destination machine to receive/display messages.
    pub destination_command: String,

    /// If `true`, the [`destination_command`][DaisyConfig::destination_command] is run for every
    /// message received. If `false`, the command is run once and remains alive to receive
    /// messages.
    pub destination_command_oneshot: bool,

    /// List of hosts which make up the chain (in order).
    pub hosts: Vec<HostSpec>,
}

impl DaisyConfig {
    pub fn load(path: &Path) -> Result<Self, Error> {
        if path == Path::new("-") {
            // Read config from stdin
            Self::load_from_stdin()
        } else {
            // Read from file
            log::debug!("Loading configuration from {}", path.display());
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        }
    }

    /// Reads configuration file from standard input
    ///
    /// The first line of input must be a format marker. The format marker is of the form
    /// `<lang>:<length>`. `<lang>` specifies the language/format of the configuration data.
    /// `<length>` specifies the amount of data to read; for text formats, it is the number of lines, and for
    /// binary formats, the number of bytes.
    ///
    /// Currently, the list of supported `<lang>` values is:
    /// - `TOML`: [TOML](https://toml.io/) format.
    pub fn load_from_stdin() -> Result<Self, Error> {
        log::debug!("Loading configuration from standard input");
        let stdin = std::io::stdin();
        let mut firstline = String::new();
        stdin.read_line(&mut firstline)?;
        // Remove trailing newline
        if &firstline[(firstline.len() - 1)..] != "\n" {
            log::error!("No newline at end of first line");
            return Err(Error::InvalidFormatMarker(firstline));
        } else {
            firstline.truncate(firstline.len() - 1);
        }

        // Read & parse based on format marker
        if let Some(lenstr) = firstline.strip_prefix("TOML:") {
            let lines: usize = lenstr.parse().map_err(|_e| {
                log::error!("Found TOML marker but failed to parse number of lines");
                Error::InvalidFormatMarker(firstline)
            })?;
            log::debug!("Reading TOML configuration from stdin");
            // Read until "EOF" line
            let mut buf = String::new();
            for _ in 0..lines {
                stdin.read_line(&mut buf)?;
            }
            return Ok(toml::from_str(&buf)?);
        }

        log::error!("Malformed or missing confguration format marker");
        Err(Error::InvalidFormatMarker(firstline))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(from = "String", into = "String")]
pub struct HostSpec {
    pub host_addr: String,
    pub username: String,
}

impl From<String> for HostSpec {
    fn from(spec: String) -> HostSpec {
        HostSpec::from(spec.as_str())
    }
}

impl From<&str> for HostSpec {
    fn from(spec: &str) -> HostSpec {
        if let Some(sep_i) = spec.find('@') {
            let host_addr = spec[(sep_i + 1)..].to_owned();
            let username = spec[..sep_i].to_owned();
            HostSpec {
                host_addr,
                username,
            }
        } else {
            HostSpec {
                host_addr: spec.to_owned(),
                username: uzers::get_current_username()
                    .expect("No username specified and unable to retrieve the current user's name")
                    .to_string_lossy()
                    .to_string(),
            }
        }
    }
}

impl From<HostSpec> for String {
    fn from(spec: HostSpec) -> String {
        format!("{}@{}", spec.username, spec.host_addr)
    }
}

/// Configuration-related error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {}", .0)]
    IOError(#[from] std::io::Error),

    #[error("TOML parse error: {}", .0)]
    TOMLError(#[from] toml::de::Error),

    #[error("Invalid format marker line. First line was \"{}\".", .0)]
    InvalidFormatMarker(String),
}
