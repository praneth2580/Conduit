use super::{InboundFrame, NetworkBackend};
use conduit_core::error::{ConduitError, Result};
use conduit_core::ids::NodeId;
use conduit_discovery::{PeerEndpoint, DEFAULT_DATA_PORT, DEFAULT_DISCOVERY_PORT};
use std::collections::{HashMap, VecDeque};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};

const DATA_MAGIC: [u8; 4] = *b"CDND";

/// LAN transport via UDP unicast to peer data ports.
#[derive(Debug)]
pub struct UdpNetwork {
  local_id: NodeId,
  socket: UdpSocket,
  peers: HashMap<NodeId, SocketAddr>,
  inbound: VecDeque<InboundFrame>,
}

impl UdpNetwork {
  pub fn new(local_id: NodeId) -> Result<Self> {
    Self::with_port(local_id, DEFAULT_DATA_PORT)
  }

  pub fn with_port(local_id: NodeId, port: u16) -> Result<Self> {
    let socket = Self::bind_socket(port)?;
    Ok(Self {
      local_id,
      socket,
      peers: HashMap::new(),
      inbound: VecDeque::new(),
    })
  }

  pub fn local_addr(&self) -> Result<SocketAddr> {
    self.socket.local_addr().map_err(ConduitError::Io)
  }

  fn bind_socket(port: u16) -> Result<UdpSocket> {
    use socket2::{Domain, Socket, Type};
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true).ok();
    socket.bind(&addr.into())?;
    socket.set_nonblocking(true)?;
    Ok(socket.into())
  }

  fn peer_data_addr(endpoint: &PeerEndpoint) -> Option<SocketAddr> {
    match endpoint {
      PeerEndpoint::Udp { addr } => {
        let port = if addr.port() == DEFAULT_DISCOVERY_PORT {
          DEFAULT_DATA_PORT
        } else {
          addr.port()
        };
        Some(SocketAddr::new(addr.ip(), port))
      }
      _ => None,
    }
  }

  fn encode_datagram(from: NodeId, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + 16 + payload.len());
    buf.extend_from_slice(&DATA_MAGIC);
    buf.extend_from_slice(from.as_bytes());
    buf.extend_from_slice(payload);
    buf
  }

  fn decode_datagram(data: &[u8]) -> Result<(NodeId, Vec<u8>)> {
    if data.len() < 4 + 16 {
      return Err(ConduitError::Deserialization("data datagram too short".into()));
    }
    if &data[..4] != DATA_MAGIC {
      return Err(ConduitError::Deserialization("invalid data datagram magic".into()));
    }
    let from = NodeId::from_bytes(
      data[4..20]
        .try_into()
        .map_err(|_| ConduitError::Deserialization("invalid node id".into()))?,
    );
    Ok((from, data[20..].to_vec()))
  }

  fn poll_socket(&mut self) -> Result<()> {
    let mut buf = [0u8; 65_536];
    loop {
      match self.socket.recv_from(&mut buf) {
        Ok((len, _addr)) => {
          let (from, data) = match Self::decode_datagram(&buf[..len]) {
            Ok(v) => v,
            Err(_) => continue,
          };
          if from == self.local_id {
            continue;
          }
          self.inbound.push_back(InboundFrame { from, data });
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
        Err(e) => return Err(ConduitError::Io(e)),
      }
    }
    Ok(())
  }
}

impl NetworkBackend for UdpNetwork {
  fn register_peer(&mut self, node_id: NodeId, endpoint: &PeerEndpoint) {
    if let Some(addr) = Self::peer_data_addr(endpoint) {
      self.peers.insert(node_id, addr);
    }
  }

  fn send_frames(&mut self, to: NodeId, frames: &[Vec<u8>]) -> Result<()> {
    let addr = self.peers.get(&to).ok_or_else(|| {
      ConduitError::Configuration(format!("no UDP endpoint for peer {to}"))
    })?;
    for frame in frames {
      let datagram = Self::encode_datagram(self.local_id, frame);
      self
        .socket
        .send_to(&datagram, addr)
        .map_err(ConduitError::Io)?;
    }
    Ok(())
  }

  fn drain_inbound(&mut self) -> Vec<InboundFrame> {
    let _ = self.poll_socket();
    self.inbound.drain(..).collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn udp_network_round_trip() {
    let id_a = NodeId::from_bytes([1u8; 16]);
    let id_b = NodeId::from_bytes([2u8; 16]);
    let mut a = UdpNetwork::with_port(id_a, 0).unwrap();
    let mut b = UdpNetwork::with_port(id_b, 0).unwrap();

    let addr_a = a.local_addr().unwrap();
    let addr_b = b.local_addr().unwrap();

    a.register_peer(id_b, &PeerEndpoint::Udp { addr: addr_b });
    b.register_peer(id_a, &PeerEndpoint::Udp { addr: addr_a });

    a.send_frames(id_b, &[b"hello".to_vec()]).unwrap();
    let frames = b.drain_inbound();
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].from, id_a);
    assert_eq!(frames[0].data, b"hello");
  }
}
