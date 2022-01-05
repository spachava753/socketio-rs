use crate::transport::*;
use axum::extract::ws::{Message, WebSocket};
use eio_parser::*;
use std::fmt::Error;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Websocket transport expects a valid SID")]
    MissingSIDWebsocket,
    #[error("Error establishing websocket connection")]
    ConnWebsocketErr(#[source] tungstenite::Error),
}

/// We will create an engine instance per request.
/// Our engine will need a transport mechanism to process the requests.
/// For websockets, the engine instance will live until the connection is closed.
/// For polling mechanisms, the instance will have to be created every request.
/// For any of the supported transports (and future transports to be implemented),
/// the engine likely needs a callback to specify what happens when a packet is received.
#[derive(Debug)]
pub struct Engine<R: Responder> {
    transport: TransportType,
    responder: R,
    sid: Option<String>,
}

impl<R: Responder> Engine<R> {
    /// The new function should be used to create a new engine instance,
    /// usually on the first request of polling transport to establish a connection
    pub fn new(transport: TransportType, responder: R) -> Engine<R> {
        Engine {
            transport,
            responder,
            sid: None,
        }
    }

    /// The `with_sid` function can used when upgrading the polling transport to websocket,
    /// or processing payloads for polling transport.
    pub fn with_sid(transport: TransportType, responder: R, sid: String) -> Engine<R> {
        Engine {
            transport,
            responder,
            sid: Some(sid),
        }
    }

    /// Currently the engine only works with axum
    async fn run(&self, stream: TcpStream) -> Result<(), EngineError> {
        match (&self.transport, &self.sid) {
            (TransportType::Websocket(t), None) => {
                Err(EngineError::MissingSIDWebsocket)
            }
            (TransportType::Websocket(t), Some(sid)) => {
                let ws_stream = accept_async(stream)
                    .await
                    .map_err(|e| EngineError::ConnWebsocketErr(e))?;

                let (write, read) = ws_stream.split();
                read.
                Ok(())
            }
            (TransportType::Polling(t), None) => {
                Ok(())
            }
            (TransportType::Polling(t), Some(sid)) => {
                Ok(())
            }
        }
    }
}

/// The trait Responder is responsible for processing each packet. It processes packet by packet
pub trait Responder {
    fn process_packet(packet: Packet);
}