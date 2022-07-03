/*
 * Server:
 * 1. messages <- listen
 * 2. replies <- mapM handle message
 *    handle Registration = writeInto map
 *    handle Resolution = readFrom map
 *    handle Purge = deleteFrom map
 *    handle Error = void $ liftIO $ print error
 */

pub enum Error {

}

pub struct NhrpHandler {

}
impl NhrpHandler {
    pub fn new() -> Self {
        Self { }
    }

    pub async fn handle_messages(&mut self, socket: &NhrpSocket) -> Result<(), Error> {

    }
}

use futures::{Future, Poll, Async, Stream, Sink, AsyncSink};
use futures_locks::RwLock;

use {Result, Error};
use super::{NhrpFramed, NhrpCodec, NhrpMessage, Operation};

use std::net::IpAddr;
use crate::NhrpSocket;

use super::services::{Registration, Peers, Purge};

pub type BoxedFuture<I> = Box<Future<Item = I, Error = Error>>;

pub trait Service: Send {
    type Request;
    type Response;
    type Future: Future<Item = Self::Response, Error = Error>;

    fn call(&mut self, Self::Request) -> Self::Future;
}
pub struct BoxedService<T, R> {
    inner: Box<Service<
        Request = T,
        Response = R,
        Future = BoxedFuture<R>,
        >>
}
impl<T, R> BoxedService<T, R> {
    pub fn new<O>(inner: O) -> Self
    where O: Service<Request = T, Response = R, Future = BoxedFuture<R>> + 'static {
        let inner = Box::new(inner);
        BoxedService { inner: inner }
    }
}
impl<T, R> Service for BoxedService<T, R> {
    type Request = T;
    type Response = R;
    type Future = BoxedFuture<R>;

    fn call(&mut self, request: Self::Request) -> Self::Future {
        self.inner.call(request)
    }
}

pub trait Routing: Send {
    type Request;
    type Response;

    type Service: Service<Request = Self::Request, Response = Self::Response>;

    fn route(&mut self, request: &Self::Request) -> Option<&mut Self::Service>;
}

pub struct Router<T> {
    routing: T
}
pub enum RouterFuture<T: Routing> {
    NotFound,
    Running(<T::Service as Service>::Future),
}

impl<T: Routing> Router<T> {
    pub fn new(routing: T) -> Router<T> {
        Router { routing: routing }
    }
}

impl<T: Routing> Service for Router<T> {
    type Request = T::Request;
    type Response = T::Response;
    type Future = RouterFuture<T>;

    fn call(&mut self, request: Self::Request) -> Self::Future {
        use self::RouterFuture::*;
        if let Some(service) = self.routing.route(&request) {
            Running(service.call(request))
        } else {
            NotFound
        }
    }
}

impl<T: Routing> Future for RouterFuture<T> {
    type Item = T::Response;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        use self::RouterFuture::*;
        match self {
            NotFound => Err(Error::NotImplemented),
            Running(f) => f.poll(),
        }
    }
}

pub struct NhrpRouting {
    registration: BoxedService<Operation, Operation>,
    purge: BoxedService<Operation, Operation>,
}

impl NhrpRouting {
    pub fn new() -> Self {
        let map = Peers::new();
        let l = RwLock::new(map);
        NhrpRouting {
            registration: BoxedService::new(Registration::new(l.clone())),
            purge: BoxedService::new(Purge::new(l.clone())),
        }
    }
}

impl Routing for NhrpRouting {
    type Request = Operation;
    type Response = Operation;
    type Service = BoxedService<Self::Request, Self::Response>;

    fn route(&mut self, request: &Self::Request) -> Option<&mut Self::Service> {
        use NhrpOp::*;
        match request.optype() {
            RegistrationRequest => Some(&mut self.registration),
            PurgeRequest => Some(&mut self.purge),
            _ => None,
        }
    }
}

pub struct ServerProto {
    transport: NhrpFramed<NhrpCodec<NhrpMessage>>,
    service: Router<NhrpRouting>,
    waiting: Option<(NhrpMessage, IpAddr)>
}

impl ServerProto {
    pub fn new(transport: NhrpFramed<NhrpCodec<NhrpMessage>>) -> ServerProto {
        ServerProto {
            transport: transport,
            service: Router::new(NhrpRouting::new()),
            waiting: None,
        }
    }
}

impl Future for ServerProto {
    type Item = ();
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // First off, check if there are still items to be sent in the pipeline
        try_ready!(self.transport.poll_complete());

        // Process as many framed as possible per tick
        loop {
            if let Some((message, addr)) = try_ready!(self.transport.poll()) {
                let (header,operation,extension) = message.into_parts();
                let f = self.service.call(operation);

                let f = f.and_then(|op| {
                    let mut responseheader = header;
                    responseheader.set_optype(op.optype());
                    let response = NhrpMessage::new(responseheader, op, Vec::new());
                    Ok(response)
                });

                let mut f = f.and_then(|r| match self.transport.start_send((r, addr)) {
                    Ok(AsyncSink::Ready) => self.transport.poll_complete(),
                    Ok(AsyncSink::NotReady(f)) => {
                        self.waiting = Some(f);
                        Ok(Async::NotReady)
                    },
                    Err(e) => Err(e),
                });

                try_ready!(f.poll());
            } else {
                return Ok(Async::Ready(()))
            }
        }
    }
}
