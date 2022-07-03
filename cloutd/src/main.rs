/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io;
use std::net::IpAddr;
use std::collections::HashMap;
use std::process::Command;
use rtnetlink::new_connection;

mod error;
mod socket;

use crate::error::CloutdError;
use crate::socket::NhrpSocket;

#[tokio::main]
async fn main() -> Result<(), miette::Error> {
    tracing_subscriber::fmt::init();
    tracing::info!("cloutd is starting");

    let (nlconn, nlhandle, answers) = new_connection()
        .map_err(CloutdError::from)?;
    tokio::spawn(nlconn);

    let nhrp_sock = NhrpSocket::new()?;

    tracing::info!(?nhrp_sock, "Opened NHRP sockets");

    Ok(())
}
