use std::mem::MaybeUninit;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::time::Duration;
use std::time::Instant;

use socket2::{Domain, Protocol, Socket, Type};

use crate::error::{PingError, Result};
use crate::packet;
use crate::packet::EchoReply;
// const SIZE: usize = 56;

pub struct Pinger {
    to_addr: IpAddr,
    socket: Socket,
    socket_addr: SocketAddr,
}

impl Pinger {
    pub fn new(to_addr: IpAddr) -> Result<Self> {
        let socket = match to_addr {
            IpAddr::V4(_) => Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4))?,
            IpAddr::V6(_) => Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6))?,
        };
        let socket_addr = SocketAddr::new(to_addr, 0);

        Ok(Self {
            to_addr,
            socket,
            socket_addr,
        })
    }

    pub fn ping(
        &self,
        ident: u16,
        seq: u16,
        size: usize,
        timeout: Duration,
    ) -> Result<(EchoReply, Duration)> {
        let mut packet = packet::encode_echo_request(self.to_addr, ident, seq, size)?;
        let send_at = Instant::now();
        self.socket
            .send_to(&mut packet, &self.socket_addr.into())
            .expect("socket send packet error");

        self.wait_reply(ident, seq, timeout)
            .map(|(reply, t)| (reply, t.duration_since(send_at)))
    }

    fn wait_reply(&self, ident: u16, seq: u16, timeout: Duration) -> Result<(EchoReply, Instant)> {
        let mut buffer = [MaybeUninit::new(0); 4096];
        let mut timeout = timeout;

        // wait the reply we want or timeout
        loop {
            self.socket.set_read_timeout(Some(timeout))?;

            let start_at = Instant::now();
            let size = self.socket.recv(&mut buffer).map_err(|e| {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    PingError::Timeout(seq)
                } else {
                    PingError::IOError(e)
                }
            })?;
            let end_at = Instant::now();

            // calculate the new timeout
            timeout = timeout - end_at.duration_since(start_at);
            if timeout <= Duration::ZERO {
                return Err(PingError::Timeout(seq));
            }

            let buf = unsafe { MaybeUninit::slice_assume_init_ref(&buffer[..size]) };

            match packet::decode_echo_reply(self.to_addr, buf) {
                Ok(reply) => {
                    if reply.ident == ident && reply.seq == seq {
                        return Ok((reply, Instant::now()));
                    }
                    continue;
                }
                Err(PingError::NotEchoReply(_)) => continue,
                Err(PingError::NotV6EchoReply(_)) => continue,
                Err(PingError::OtherICMP) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}
