/*
 * Server:
 * 1. messages <- listen
 * 2. replies <- mapM handle message
 *    handle Registration = writeInto map
 *    handle Resolution = readFrom map
 *    handle Purge = deleteFrom map
 *    handle Error = void $ liftIO $ print error
 */

use std::collections::HashMap;

pub type Server = HashMap<IpAddr, Peer>

pub struct Peer {
    
}
