/*
 * Server:
 * 1. messages <- listen
 * 2. replies <- mapM handle message
 *    handle Registration = writeInto map
 *    handle Resolution = readFrom map
 *    handle Purge = deleteFrom map
 *    handle Error = void $ liftIO $ print error
 */

use std::io;
use thiserror::Error;
use miette::Diagnostic;
use nhrp::{NhrpBuffer, NhrpOp, OperationBuffer};
use crate::{NhrpSocket, socket};

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("socket error occurred")]
    Socket(#[source] #[from] #[diagnostic_source] socket::Error),
    #[error("received invalid NHRP message")]
    Parse(#[source] #[from] nhrp::Error),
    #[error("received msg with unknown operation type {0}")]
    UnknownOpType(u8),
}

const BUFFER_LEN: usize = 2048;

pub struct NhrpHandler {

}
impl NhrpHandler {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn on_resolution_request(&self, msg: NhrpBuffer<&'_ [u8]>) -> Result<(), Error> {
        Ok(())
    }

    pub async fn handle_messages(&mut self, socket: &NhrpSocket) -> Result<(), Error> {
        loop {
            let mut buf = [0u8; BUFFER_LEN];
            let (len, source) = socket.recv(&mut buf).await?;
            if len == 0 {
                return Ok(());
            }
            let msgbuf = &buf[0..len];
            let msg = NhrpBuffer::new_checked(msgbuf)?;
            match msg.optype() {
                NhrpOp::ResolutionRequest => {
                    self.on_resolution_request(msg).await?;
                }
                NhrpOp::ResolutionReply => {}
                NhrpOp::RegistrationRequest => {}
                NhrpOp::RegistrationReply => {}
                NhrpOp::PurgeRequest => {}
                NhrpOp::PurgeReply => {}
                NhrpOp::ErrorIndication => {}
                NhrpOp::Other(val) => {
                    return Err(Error::UnknownOpType(val));
                }
            }
        }
    }
}