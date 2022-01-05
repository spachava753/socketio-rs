use eio_parser::*;
use thiserror::Error;

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TransportParsingError {
    #[error("Encountered a packet parsing error")]
    PacketParsingErr(#[source] PacketParsingError),
    #[error("Expected single packet, but received {0}")]
    InvalidPayloadForWebsocket(usize),
    #[error("Received pong packet with data")]
    InvalidPongPacket,
    #[error("Received ping packet from client")]
    InvalidPingPacket,
}

#[derive(Debug)]
pub enum TransportType {
    Websocket(WebsocketTransport),
    Polling(PollingTransport),
}

pub trait Transport {
    fn parse_payload(&self, payload_msg: &str) -> Result<Payload, TransportParsingError>;
}

#[derive(Debug)]
pub struct WebsocketTransport;

impl Transport for WebsocketTransport {
    // when upgrading from transport polling transport, client sends a ping packet with data "probe"
    // e.g. "2probe". Server is supposed to respond with 3probe. From then on, the server is only
    // one who sends the ping packet with no data e.g. "2", while the client can only respond with
    // the pong packet e.g. "3"
    fn parse_payload(&self, payload_msg: &str) -> Result<Payload, TransportParsingError> {
        match Payload::try_from(payload_msg) {
            Ok(payload) => {
                if payload.len() > 1 {
                    Err(TransportParsingError::InvalidPayloadForWebsocket(
                        payload.len(),
                    ))
                } else {
                    Ok(payload)
                }
            }
            Err(parsing_err) => Err(TransportParsingError::PacketParsingErr(parsing_err)),
        }
    }
}

#[derive(Debug)]
pub struct PollingTransport;

impl Transport for PollingTransport {
    fn parse_payload(&self, payload_msg: &str) -> Result<Payload, TransportParsingError> {
        match Payload::try_from(payload_msg) {
            Ok(payload) => {
                for p in payload.packets() {
                    match p.get_packet_type() {
                        PacketType::Pong => {
                            // check that packet has no data
                            if let Some(_) = p.get_packet_data() {
                                return Err(TransportParsingError::InvalidPongPacket);
                            }
                        }
                        PacketType::Ping => {
                            // we are not supposed to receive ping packets from client
                            if let Some(_) = p.get_packet_data() {
                                return Err(TransportParsingError::InvalidPingPacket);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(payload)
            }
            Err(parsing_err) => Err(TransportParsingError::PacketParsingErr(parsing_err)),
        }
    }
}
