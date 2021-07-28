use std::net::IpAddr;

use crate::error::Result;

mod v4;
mod v6;

#[derive(Debug)]
pub struct EchoPacket {
    pub to_addr: IpAddr,
    pub ident: u16,
    pub seq: u16,
    pub size: usize,
}

impl EchoPacket {
    fn new(to_addr: IpAddr, ident: u16, seq: u16, size: usize) -> Self {
        Self {
            to_addr,
            ident,
            seq,
            size,
        }
    }
}

#[derive(Debug)]
pub struct EchoReply {
    pub ttl: Option<u8>,
    pub source: IpAddr,
    pub seq: u16,
    pub ident: u16,
    pub size: usize,
}

pub trait EchoEncoder {
    fn encode(p: EchoPacket) -> Result<Vec<u8>>;
}

pub trait EchoDecoder {
    fn decode(addr: IpAddr, buf: &[u8]) -> Result<EchoReply>;
}

// Dispather

pub fn encode_echo_request(to_addr: IpAddr, ident: u16, seq: u16, size: usize) -> Result<Vec<u8>> {
    let packet = EchoPacket::new(to_addr, ident, seq, size);
    if to_addr.is_ipv4() {
        v4::IPv4EchoEncoder::encode(packet)
    } else {
        v6::IPv6EchoEncoder::encode(packet)
    }
}

pub fn decode_echo_reply(addr: IpAddr, buf: &[u8]) -> Result<EchoReply> {
    if addr.is_ipv4() {
        v4::IPv4EchoDecoder::decode(addr, buf)
    } else {
        v6::IPv6EchoDecoder::decode(addr, buf)
    }
}
