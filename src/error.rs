use std::io;

use pnet_packet::{icmp::IcmpType, icmpv6::Icmpv6Type};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PingError>;

#[derive(Error, Debug)]
pub enum PingError {
    #[error("buffer size was too small")]
    IncorrectBufferSize,
    #[error("invalid packet: {0}")]
    InvalidPacket(#[from] InvalidPacketError),
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("expected echoreply, got {0:?}")]
    NotEchoReply(IcmpType),
    #[error("expected echoreply, got {0:?}")]
    NotV6EchoReply(Icmpv6Type),
    #[error("Request timeout for icmp_seq {0}")]
    Timeout(u16),
    #[error("other icmp message")]
    OtherICMP,
}

#[derive(Error, Debug)]
pub enum InvalidPacketError {
    #[error("expected an Ipv4Packet")]
    NotIpv4Packet,
    #[error("expected an IcmpPacket payload")]
    NotIcmpPacket,
    #[error("expected an Icmpv6Packet")]
    NotIcmpv6Packet,
    #[error("payload too short, got {got}, want {want}")]
    PayloadTooShort { got: usize, want: usize },
}
