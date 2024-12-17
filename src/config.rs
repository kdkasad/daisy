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

#[derive(Debug, PartialEq, Clone)]
pub struct HostSpec {
    pub host_addr: String,
    pub username: String,
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
