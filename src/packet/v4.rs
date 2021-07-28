use std::net::IpAddr;

use pnet_packet::icmp::checksum;
use pnet_packet::icmp::echo_reply::EchoReplyPacket;
use pnet_packet::icmp::echo_request::MutableEchoRequestPacket;
use pnet_packet::icmp::{IcmpPacket, IcmpTypes};
use pnet_packet::ipv4::Ipv4Packet;
use pnet_packet::Packet;

use super::{EchoDecoder, EchoEncoder};
use super::{EchoPacket, EchoReply};
use crate::error::{InvalidPacketError, PingError, Result};

pub struct IPv4EchoEncoder {}

impl EchoEncoder for IPv4EchoEncoder {
    fn encode(p: EchoPacket) -> Result<Vec<u8>> {
        // header occupy 8 bytes.
        let size = 8 + p.size;
        let mut buf = vec![0; size];
        let mut packet =
            MutableEchoRequestPacket::new(&mut buf[..]).ok_or(PingError::IncorrectBufferSize)?;
        packet.set_icmp_type(IcmpTypes::EchoRequest);
        packet.set_identifier(p.ident);
        packet.set_sequence_number(p.seq);

        let icmp_packet = IcmpPacket::new(packet.packet()).ok_or(PingError::IncorrectBufferSize)?;
        let checksum = checksum(&icmp_packet);
        packet.set_checksum(checksum);

        Ok(packet.packet().to_vec())
    }
}

pub struct IPv4EchoDecoder {}

impl EchoDecoder for IPv4EchoDecoder {
    fn decode(addr: IpAddr, buf: &[u8]) -> Result<EchoReply> {
        let ipv4_packet = Ipv4Packet::new(buf)
            .ok_or_else(|| PingError::from(InvalidPacketError::NotIpv4Packet))?;

        let payload = ipv4_packet.payload();
        let icmp_packet = IcmpPacket::new(payload)
            .ok_or_else(|| PingError::from(InvalidPacketError::NotIcmpPacket))?;
        let typ = icmp_packet.get_icmp_type();
        if typ != IcmpTypes::EchoReply {
            return Err(PingError::NotEchoReply(typ));
        }

        let echo_reply_packet = EchoReplyPacket::new(payload).unwrap();
        Ok(EchoReply {
            ttl: Some(ipv4_packet.get_ttl()),
            source: addr,
            seq: echo_reply_packet.get_sequence_number(),
            ident: echo_reply_packet.get_identifier(),
            size: echo_reply_packet.packet().len(),
        })
    }
}
