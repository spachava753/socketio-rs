use base64::DecodeError;
use thiserror::Error;

const PACKET_SEPARATOR: &str = "\x1e";
const PACKET_PROBE: &str = "probe";

#[derive(Error, Debug, Eq, PartialEq)]
pub enum PacketParsingError {
    #[error("invalid char")]
    InvalidChar,
    #[error("Invalid packet length")]
    InvalidPacketLen,
    #[error("Emtpy string")]
    EmptyString,
    #[error("Invalid Binary Message")]
    InvalidBinaryMessage,
    /// An invalid ping occurs when we are using the XHR transport and we get anything else besides '2probe'
    #[error("invalid ping packet")]
    InvalidPing,
    /// An invalid pong occurs when we are using the XHR transport and we get anything else besides '3probe'
    #[error("invalid pong packet")]
    InvalidPong,
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
                '2' => {
                    let msg = chars.collect::<String>();
                    if msg.len() > 0 && msg != PACKET_PROBE {
                        Err(PacketParsingError::InvalidPing)
                    } else {
                        Ok(Packet {
                            packet_type: PacketType::Ping,
                            data: Some(PacketData::String(msg)),
                        })
                    }
                }
                '3' => {
                    let msg = chars.collect::<String>();
                    if msg.len() > 0 && msg != PACKET_PROBE {
                        Err(PacketParsingError::InvalidPong)
                    } else {
                        Ok(Packet {
                            packet_type: PacketType::Pong,
                            data: Some(PacketData::String(msg)),
                        })
                    }
                }
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
#[derive(Debug, Eq, PartialEq)]
pub struct Payload {
    packets: Vec<Packet>,
}

impl Payload {
    pub fn len(&self) -> usize {
        self.packets.len()
    }
}

impl TryFrom<&str> for Payload {
    type Error = PacketParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut payload = Payload {
            packets: Vec::new()
        };
        for packet_str in value.split(PACKET_SEPARATOR) {
            payload.packets.push(Packet::try_from(packet_str)?);
        }
        Ok(payload)
    }
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

    #[test]
    fn hello_message_payload() {
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::String("hello".to_string())),
            }]
        }, "4hello".try_into().unwrap());
    }

    #[test]
    fn binary_message_payload() {
        let mut base64_msg = "b".to_string();
        base64_msg.push_str(base64::encode(vec![1, 2, 3]).as_str());
        println!("base64 encoded message: {}", base64_msg);
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::Binary(vec![1, 2, 3])),
            }]
        }, base64_msg.as_str().try_into().unwrap());
    }

    #[test]
    fn multi_message_payload() {
        let mut payload_msg = "4hello".to_string();
        payload_msg.push_str(PACKET_SEPARATOR);
        payload_msg.push_str("4world");
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::String("hello".to_string())),
            }, Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::String("world".to_string())),
            }]
        }, payload_msg.as_str().try_into().unwrap());
    }

    #[test]
    fn multi_message_binary_payload() {
        let mut payload_msg = "4hello".to_string();
        payload_msg.push_str(PACKET_SEPARATOR);
        let base64_msg = base64::encode(vec![1, 2, 3]);
        println!("base64 encoded message: {}", base64_msg);
        payload_msg.push_str("b");
        payload_msg.push_str(base64_msg.as_str());
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::String("hello".to_string())),
            }, Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::Binary(vec![1, 2, 3])),
            }]
        }, payload_msg.as_str().try_into().unwrap());
    }

    #[test]
    fn blank_packet_in_payload() {
        let mut payload_msg = "4hello".to_string();
        payload_msg.push_str(PACKET_SEPARATOR);
        payload_msg.push_str(PACKET_SEPARATOR);
        let base64_msg = base64::encode(vec![1, 2, 3]);
        println!("base64 encoded message: {}", base64_msg);
        payload_msg.push_str("b");
        payload_msg.push_str(base64_msg.as_str());
        assert_eq!(Err(PacketParsingError::EmptyString), Payload::try_from(payload_msg.as_str()));
    }

    #[test]
    fn single_packet_in_payload() {
        let mut payload_msg = "4hello".to_string();
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Message,
                data: Some(PacketData::String("hello".to_string())),
            }]
        }, Payload::try_from(payload_msg.as_str()).unwrap());
    }

    #[test]
    fn probe_ping_packet() {
        let mut payload_msg = "2probe".to_string();
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Ping,
                data: Some(PacketData::String("probe".to_string())),
            }]
        }, Payload::try_from(payload_msg.as_str()).unwrap());
    }

    #[test]
    fn probe_pong_packet() {
        let mut payload_msg = "3probe".to_string();
        assert_eq!(Payload {
            packets: vec![Packet {
                packet_type: PacketType::Pong,
                data: Some(PacketData::String("probe".to_string())),
            }]
        }, Payload::try_from(payload_msg.as_str()).unwrap());
    }
}
