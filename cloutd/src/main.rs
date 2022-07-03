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

mod socket;
mod kernel;
mod error;

use crate::socket::NhrpSocket;

#[tokio::main]
async fn main() -> Result<(), miette::Error> {
    miette::set_hook(Box::new(|_| {
        Box::new(miette::MietteHandlerOpts::new()
            .terminal_links(true)
            .unicode(true)
            .color(true)
            .rgb_colors(miette::RgbColors::Preferred)
            .context_lines(3)
            .build())
    }));

    tracing_subscriber::fmt::init();
    tracing::info!("cloutd is starting");

    let (nlconn, nlhandle, answers) = new_connection()
        .map_err(kernel::Error::ConnectionError)?;
    tokio::spawn(nlconn);

    let nhrp_sock = NhrpSocket::new()?;

    tracing::info!(?nhrp_sock, "Opened NHRP sockets");

    Ok(())
}
