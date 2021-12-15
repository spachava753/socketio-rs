use base64::DecodeError;
use thiserror::Error;

const PACKET_SEPARATOR: &str = "\x1e";

#[derive(Error, Debug)]
pub enum PacketParsingError {
    #[error("invalid char")]
    InvalidChar,
    #[error("Invalid packet length")]
    InvalidPacketLen,
    #[error("Emtpy string")]
    EmptyString,
    #[error("Invalid Binary Message")]
    InvalidBinaryMessage,
}

/// Packet type can one of enumerations
#[derive(Debug, Eq, PartialEq)]
pub enum PacketType {
    Open,
    Close,
    Ping,
    Pong,
    Message,
    Upgrade,
    Noop,
}

/// Packet data can be UTF-8 string or binary data
#[derive(Debug, Eq, PartialEq)]
pub enum PacketData {
    String(String),
    Binary(Vec<u8>),
}

/// A packet has a packet type, and some optional data
#[derive(Debug, Eq, PartialEq)]
pub struct Packet {
    packet_type: PacketType,
    data: Option<PacketData>,
}

impl TryFrom<&str> for Packet {
    type Error = PacketParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() < 1 {
            return Err(PacketParsingError::EmptyString);
        }
        let mut chars = value.chars();
        if let Some(ch) = chars.next() {
            match ch {
                '0' => Ok(Packet {
                    packet_type: PacketType::Open,
                    data: None,
                }),
                '1' => Ok(Packet {
                    packet_type: PacketType::Close,
                    data: None,
                }),
                '2' => Ok(Packet {
                    packet_type: PacketType::Ping,
                    data: None,
                }),
                '3' => Ok(Packet {
                    packet_type: PacketType::Pong,
                    data: None,
                }),
                '4' => Ok(Packet {
                    packet_type: PacketType::Message,
                    data: Some(PacketData::String(chars.collect::<String>())),
                }),
                'b' => {
                    let bytes = chars.collect::<String>().into_bytes();
                    match base64::decode(bytes) {
                        Ok(b) => Ok(Packet {
                            packet_type: PacketType::Message,
                            data: Some(PacketData::Binary(b)),
                        }),
                        Err(_) => Err(PacketParsingError::InvalidBinaryMessage)
                    }
                }
                '5' => Ok(Packet {
                    packet_type: PacketType::Upgrade,
                    data: None,
                }),
                '6' => Ok(Packet {
                    packet_type: PacketType::Noop,
                    data: None,
                }),
                _ => Err(PacketParsingError::InvalidChar)
            }
        } else {
            Err(PacketParsingError::InvalidChar)
        }
    }
}

/// A payload is composed of one or more packets
struct Payload {
    packets: Vec<Packet>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_message() {
        assert_eq!(Packet {
            packet_type: PacketType::Message,
            data: Some(PacketData::String("hello".to_string())),
        }, "4hello".try_into().unwrap());
    }

    #[test]
    fn binary_message() {
        let mut base64_msg = "b".to_string();
        base64_msg.push_str(base64::encode(vec![1, 2, 3]).as_str());
        println!("base64 encoded message: {}", base64_msg);
        assert_eq!(Packet {
            packet_type: PacketType::Message,
            data: Some(PacketData::Binary(vec![1, 2, 3])),
        }, base64_msg.as_str().try_into().unwrap());
    }
}
