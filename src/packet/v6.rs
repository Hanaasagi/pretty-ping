use std::{convert::TryInto, net::IpAddr};

use pnet_packet::icmpv6::{Icmpv6Packet, Icmpv6Types, MutableIcmpv6Packet};
use pnet_packet::Packet;

use super::{EchoDecoder, EchoEncoder};
use super::{EchoPacket, EchoReply};
use crate::error::{InvalidPacketError, PingError, Result};

pub struct IPv6EchoEncoder {}

impl EchoEncoder for IPv6EchoEncoder {
    fn encode(p: EchoPacket) -> Result<Vec<u8>> {
        // 4 bytes ICMP header + 2 bytes ident + 2 bytes sequence, then payload
        let size = 4 + 2 + 2 + p.size;
        let mut buf = vec![0u8; size];
        let mut packet =
            MutableIcmpv6Packet::new(&mut buf[..]).ok_or(PingError::IncorrectBufferSize)?;
        packet.set_icmpv6_type(Icmpv6Types::EchoRequest);

        let mut payload = vec![0; 4];
        payload[0..2].copy_from_slice(&p.ident.to_be_bytes()[..]);
        payload[2..4].copy_from_slice(&p.seq.to_be_bytes()[..]);
        packet.set_payload(&payload);

        // no checksum in ipv6
        Ok(packet.packet().to_vec())
    }
}

pub struct IPv6EchoDecoder {}

impl EchoDecoder for IPv6EchoDecoder {
    fn decode(addr: IpAddr, buf: &[u8]) -> Result<EchoReply> {
        let icmp_packet = Icmpv6Packet::new(buf)
            .ok_or_else(|| PingError::from(InvalidPacketError::NotIcmpv6Packet))?;
        let typ = icmp_packet.get_icmpv6_type();
        if typ != Icmpv6Types::EchoReply {
            return Err(PingError::NotV6EchoReply(typ));
        }

        let payload = icmp_packet.payload();
        if payload.len() < 4 {
            return Err(InvalidPacketError::PayloadTooShort {
                got: payload.len(),
                want: 4,
            }
            .into());
        }
        let ident = u16::from_be_bytes(payload[0..2].try_into().unwrap());
        let seq = u16::from_be_bytes(payload[2..4].try_into().unwrap());

        Ok(EchoReply {
            ttl: None,
            source: addr,
            seq,
            ident,
            size: payload.len() - 4, // Subtract 4 bytes for ident and sequence
        })
    }
}
