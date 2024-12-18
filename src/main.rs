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

use std::path::PathBuf;

use clap::Parser;
use daisy::{config::DaisyConfig, worm::infect};
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode};

#[derive(Parser)]
#[command(version, about, author, long_about = None)]
#[command(next_line_help = true)] // Display option descriptions on a new line
struct CLIOptions {
    /// Configuration file path.
    #[arg(short = 'f', long, value_name = "FILE", default_value = "daisy.toml")]
    config_file: PathBuf,

    /// Logging verbosity.
    /// Specify multiple times to increase verbosity.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    // Parse CLI options
    let cli = CLIOptions::parse();

    // Initialize logging system
    simplelog::TermLogger::init(
        match cli.verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        },
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .expect("Failed to initialize logging system");

    // Parse config file
    let config = match DaisyConfig::load(&cli.config_file) {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load configuration: {}", err);
            return;
        }
    };

    // If at the destination, print the configuration object.
    // Otherwise, connect to the next host.
    if config.link_index >= config.hosts.len() {
        println!("{:?}", &config);
    } else {
        infect(config.hosts.first().unwrap(), &config).unwrap();
    }
}
