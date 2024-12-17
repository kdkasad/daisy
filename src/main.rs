/*
 * src/main.rs - Daisy - A ridiculous SSH daisy chain
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

use clap::{arg, crate_authors, crate_description, crate_name};
use daisy::{worm::infect, config::HostSpec};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};

fn main() {
    simplelog::TermLogger::init(
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to initialize logging system");

    let args = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(arg!(<hosts> ... "A list of hosts to connect to."))
        .get_matches();

    let hosts: Vec<&String> = args.get_many::<String>("hosts").unwrap().collect::<_>();
    println!("{:?}", hosts);
    if hosts.len() < 1 {
        eprintln!("Error: At least one host required.");
    }

    infect(&HostSpec::from(hosts[0].as_str())).unwrap();
}
