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

use std::{io::Read, path::Path};

use serde::{Deserialize, Serialize};

/// Daisy configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DaisyConfig {
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
        let contents = if path == Path::new("-") {
            // Read config from stdin
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        } else {
            // Read from file
            std::fs::read_to_string(path)?
        };
        Ok(toml::from_str(&contents)?)
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

//// Configuration-related error type
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error: {}", .0)]
    IOError(#[from] std::io::Error),

    #[error("TOML parse error: {}", .0)]
    ParseError(#[from] toml::de::Error),
}
