/*
 * src/worm.rs - Daisy - A ridiculous SSH daisy chain
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

//!
//! # Worm module
//!
//! Handles propagating the chain to the next step.
//!

use std::{
    io::{ErrorKind, Read, Write},
    path::Path,
};

use libssh2_sys::LIBSSH2_ERROR_CHANNEL_REQUEST_DENIED;

use crate::config::HostSpec;

/// # Chain link
/// Represents a handle on the connection to the next link.
pub struct ChainLink {
    ssh_session: ssh2::Session,
    ssh_channel: ssh2::Channel,
}

pub fn infect(host: &HostSpec) -> Result<ChainLink, Error> {
    // Establish TCP connection
    log::trace!("Connecting to {}", &host.host_addr);
    let conn =
        std::net::TcpStream::connect(&host.host_addr).map_err(Error::ConnectionFailed)?;

    // Create SSH session.
    let mut session = ssh2::Session::new().map_err(Error::SSHPreauthError)?;
    session.set_compress(true);
    session.set_blocking(true);

    // Perform SSH handshake
    log::trace!("Beginning SSH handshake");
    session.set_tcp_stream(conn);
    session.handshake().map_err(Error::SSHPreauthError)?;
    log::trace!("SSH handshake complete");

    // Perform authentication
    log::trace!("Beginning SSH authentication");
    let mut agent = session
        .agent()
        .expect("Failed to open a handle on the SSH agent");
    agent.connect().expect("Failed to connect to an SSH agent");
    agent
        .list_identities()
        .expect("Failed to get a list of keys from the SSH agent");
    let keys = agent
        .identities()
        .expect("Failed to get a list of keys from the SSH agent");
    let mut authenticated: bool = false;
    for key in keys {
        if agent.userauth(&host.username, &key).is_ok() {
            authenticated = true;
            break;
        }
    }
    if !authenticated {
        log::error!("SSH authentication failed");
        return Err(Error::SSHAuthenticationFailed(
            ssh2::Error::last_session_error(&session).unwrap(),
        ));
    }
    log::trace!("SSH authentication complete");

    // Upload executable
    let remote_exe_path = match upload_executable_scp(&mut session) {
        Ok(path) => path,
        Err(err) => {
            log::error!("SCP: {}", err);
            log::warn!("SCP file upload failed. Falling back to shell commands.");
            upload_executable_printf(&mut session)?
        }
    };
    log::trace!("Executable uploaded as {}", &remote_exe_path);

    // Execute uploaded binary
    let mut channel = session
        .channel_session()
        .map_err(Error::ExecuteDaisy)?;
    channel
        .exec(&remote_exe_path)
        .map_err(Error::ExecuteDaisy)?;
    log::trace!("Executed Daisy on remote host");

    // DEBUG: Read command output. This is just to verify that the command was executed.
    let mut buf: Vec<u8> = Vec::new();
    channel.read_to_end(&mut buf).unwrap();
    std::io::stdout().write_all(&buf).unwrap();
    channel.stderr().read_to_end(&mut buf).unwrap();
    std::io::stdout().write_all(&buf).unwrap();
    channel.wait_eof().unwrap();

    Ok(ChainLink {
        ssh_session: session,
        ssh_channel: channel,
    })
}

/// Opens a shell channel in the given SSH session
fn spawn_shell(session: &mut ssh2::Session) -> Result<ssh2::Channel, ssh2::Error> {
    let mut channel = session.channel_session()?;
    if let Err(err) = channel.exec("/bin/sh") {
        log::warn!("Executing /bin/sh failed. Falling back to login shell.");
        // If execution of the command fails, try a regular shell
        if let ssh2::ErrorCode::Session(LIBSSH2_ERROR_CHANNEL_REQUEST_DENIED) = err.code() {
            channel.shell()?;
        }
    }
    Ok(channel)
}

/// Uploads the executable to the remote host using SCP.
///
/// Returns the path to the file on the remote host.
fn upload_executable_scp(session: &mut ssh2::Session) -> Result<String, UploadExecutableError> {
    log::trace!("Uploading executable over SCP");

    // Upload bytes over SCP
    let exe_bytes = read_current_exe()?;
    let pathname = "/tmp/daisy".to_string();
    let path = Path::new(&pathname);
    let mut scp = session.scp_send(path, 0o700, exe_bytes.len() as u64, None)?;
    scp.write_all(&exe_bytes)?;
    scp.send_eof()?;
    scp.wait_eof()?;
    scp.close()?;
    scp.wait_close()?;
    log::trace!("Uploading file finished");
    Ok(pathname)
}

/// Uploads the executable to the remote host by sending a series of `printf(1)` commands
///
/// Returns the path to the file on the remote host.
fn upload_executable_printf(session: &mut ssh2::Session) -> Result<String, UploadExecutableError> {
    log::trace!("Uploading executable using shell commands");

    let mut channel = spawn_shell(session)?;

    writeln!(channel, "#!/bin/sh")?;
    writeln!(channel, "set -e")?;
    writeln!(channel, "set -o pipefail")?;
    writeln!(channel, "umask 077")?;

    // Redirect stdout to a temporary file while keeping the original stream
    writeln!(channel, "file=\"$(mktemp || echo /tmp/daisy)\"")?;
    writeln!(channel, "exec 3>&1")?;
    writeln!(channel, "exec 1>\"$file\"")?;

    // Send the bytes of the executable as a bunch of printf(1) commands
    let exe_bytes = read_current_exe()?;
    log::trace!("Sending {} bytes...", exe_bytes.len());
    const MAX_LINE_LEN: usize = 2048;
    const BYTES_PER_LINE: usize = (MAX_LINE_LEN - "printf ''".len()) / "\\x00".len();
    for line in exe_bytes.chunks(BYTES_PER_LINE) {
        write!(channel, "printf '")?;
        for byte in line {
            write!(channel, "\\x{:02x}", byte)?;
        }
        writeln!(channel, "'")?;
    }

    log::trace!("Sent all executable file data");

    // Reset stdout to the original stream
    writeln!(channel, "exec 1>&3")?;
    writeln!(channel, "exec 3>&-")?;

    // Make saved file executable
    writeln!(channel, "chmod o+x \"$file\"")?;

    // Use non-blocking I/O
    let is_blocking = session.is_blocking();
    session.set_blocking(false);

    // Discard all data currently buffered on stdout
    log::trace!("Discarding data received up until now...");
    let mut bytes_discarded: usize = 0;
    let mut buf = [0u8; 4096];
    loop {
        match channel.read(&mut buf) {
            Ok(0) => break,
            Err(err) if err.kind() == ErrorKind::WouldBlock => break,
            Ok(chunksize) => bytes_discarded += chunksize,
            Err(err) if err.kind() == ErrorKind::Interrupted => (),
            Err(err) => return Err(err)?,
        }
        log::debug!("read complete");
    }
    log::trace!("Discarded {} bytes", bytes_discarded);

    // Send command to print temporary file name
    log::trace!("Attempting to get uploaded file path");
    let mut buf: Vec<u8> = Vec::with_capacity(1024);
    writeln!(channel, "printf '%s' \"$file\"")?;
    channel.flush()?;
    channel.send_eof()?;
    session.set_blocking(true);
    channel.wait_eof()?;
    channel.read_to_end(&mut buf)?;
    let filename: String = String::from_utf8_lossy(&buf).to_string();

    // Restore previous blocking state
    session.set_blocking(is_blocking);

    // Close shell channel
    channel.close()?;
    channel.wait_close()?;

    Ok(filename)
}

/// Reads the contents of the current process' executable and returns the bytes.
fn read_current_exe() -> Result<Vec<u8>, std::io::Error> {
    let exe_path =
        std::env::current_exe()?;
    let exe_bytes: Vec<u8> =
        std::fs::read(&exe_path)?;
    log::trace!("Found current executable at path {}", exe_path.to_string_lossy());
    Ok(exe_bytes)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connection to remote host failed")]
    ConnectionFailed(#[source] std::io::Error),

    #[error("SSH error occurred before authentication")]
    SSHPreauthError(#[source] ssh2::Error),

    #[error("SSH authentication failed")]
    SSHAuthenticationFailed(#[source] ssh2::Error),

    #[error("Failed to spawn shell on remote host")]
    SpawnShell(#[source] ssh2::Error),

    #[error("Failed to upload executable to remote host")]
    UploadExecutable(#[from] UploadExecutableError),

    #[error("Failed to execute uploaded Daisy binary")]
    ExecuteDaisy(#[source] ssh2::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum UploadExecutableError {
    #[error("I/O operation failed: {}", .0)]
    IO(#[from] std::io::Error),

    #[error("SSH operation failed: {}", .0)]
    SSH(#[from] ssh2::Error),
}
