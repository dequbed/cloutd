use std::io;
use bytes::{Bytes, BytesMut};
use nhrp::{Emitable, NhrpBuffer, NhrpMessage};
use thiserror::Error;
use miette::Diagnostic;
use crate::NhrpSocket;